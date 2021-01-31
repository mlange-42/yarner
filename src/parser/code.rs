//! The parser for code files.

use super::md::MdParser;

use crate::config::BlockLabels;

/// Representation of a code block
pub struct RevCodeBlock {
    /// Doc source file
    pub file: String,
    /// Block name
    pub name: Option<String>,
    /// Block index
    pub index: usize,
    /// Text lines
    pub lines: Vec<String>,
    /// Indent
    pub indent: String,
}

impl RevCodeBlock {
    fn new(file: String, name: Option<String>, index: usize, indent: String) -> Self {
        Self {
            file,
            name,
            index,
            lines: vec![],
            indent,
        }
    }
    fn push_line(&mut self, line: String) {
        self.lines.push(line);
    }
}

/// Parses code files
pub struct CodeParser {}

impl CodeParser {
    /// Parse a code file in a specified language
    pub fn parse(
        &self,
        source: &str,
        parser: &MdParser,
        block_labels: &BlockLabels,
    ) -> Result<Vec<RevCodeBlock>, String> {
        let start = format!(
            "{} {}",
            block_labels.comment_start, block_labels.block_start
        );
        let next = format!("{} {}", block_labels.comment_start, block_labels.block_next);
        let end = format!("{} {}", block_labels.comment_start, block_labels.block_end);

        let mut blocks = vec![];
        let mut block_stack: Vec<RevCodeBlock> = vec![];
        for line in source.lines() {
            let trimmed = line.trim_start();
            let indent = line.len() - trimmed.len();
            if trimmed.starts_with(&start) || trimmed.starts_with(&next) {
                let is_next = trimmed.starts_with(&next);
                if is_next {
                    let block = block_stack.pop();
                    if let Some(block) = block {
                        blocks.push(block);
                    }
                }

                let mut full_name = if is_next {
                    trimmed[next.len()..].trim().to_string()
                } else {
                    trimmed[start.len()..].trim().to_string()
                };
                if let Some(comment_end) = &block_labels.comment_end {
                    if let Some(idx) = full_name.find(comment_end) {
                        full_name = full_name[..idx].trim().to_string();
                    }
                }

                let mut parts = full_name.splitn(3, &block_labels.name_separator);
                let file = parts.next().unwrap_or("").to_string();
                let name = parts.next().and_then(|s| {
                    if s.is_empty() {
                        None
                    } else {
                        Some(s.to_string())
                    }
                });
                let index_str = parts.next().unwrap_or("0");
                let index = index_str.parse::<usize>().map_err(|_| {
                    format!("Can't parse block index '{}' to an integer", index_str)
                })?;

                if !is_next {
                    if let (Some(name), Some(block)) = (&name, block_stack.last_mut()) {
                        let new_line = format!(
                            "{}{}{}{}{}",
                            &line[..indent],
                            parser.macro_start,
                            if parser.macro_start.ends_with(' ') {
                                ""
                            } else {
                                " "
                            },
                            name,
                            parser.macro_end,
                        );
                        block.push_line(new_line);
                    }
                }

                let block =
                    RevCodeBlock::new(file, name.clone(), index, line[..indent].to_string());
                block_stack.push(block);
            } else if trimmed.starts_with(&end) {
                let block = block_stack.pop();
                if let Some(block) = block {
                    blocks.push(block);
                }
            } else if let Some(block) = block_stack.last_mut() {
                if line.starts_with(&block.indent) {
                    block.push_line(line[block.indent.len()..].to_string());
                } else {
                    block.push_line(line.to_string());
                }
            }
        }

        Ok(blocks)
    }
}
