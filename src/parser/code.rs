//! The parser for code files.

use crate::config::LanguageSettings;
use crate::parser::ParserConfig;
use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::collections::HashMap;

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
    pub fn parse<P: ParserConfig>(
        &self,
        source: &str,
        parser: &P,
        language: &LanguageSettings,
    ) -> Vec<RevCodeBlock> {
        let start = format!("{} {}", language.comment_start, language.block_start);
        let end = format!("{} {}", language.comment_start, language.block_end);

        let mut block_count: HashMap<String, usize> = HashMap::new();
        let mut blocks = vec![];
        let mut block_stack: Vec<RevCodeBlock> = vec![];
        for line in source.lines() {
            let trimmed = line.trim_start();
            let indent = line.len() - trimmed.len();
            if trimmed.starts_with(&start) {
                let full_name = trimmed[start.len()..].to_string();
                let mut parts = full_name.splitn(2, '#');
                let file = parts.next().unwrap_or("").to_string();
                let name = parts.next().and_then(|s| {
                    if s.is_empty() {
                        None
                    } else {
                        Some(s.to_string())
                    }
                });

                if let (Some(name), Some(block)) = (&name, block_stack.last_mut()) {
                    let new_line = format!(
                        "{}{} {}{}",
                        " ".repeat(indent),
                        parser.macro_start(),
                        name,
                        parser.macro_end()
                    );
                    block.push_line(new_line);
                }

                let index = match block_count.entry(full_name) {
                    Occupied(mut entry) => entry.insert(*entry.get() + 1),
                    Vacant(entry) => {
                        entry.insert(1);
                        0
                    }
                };

                let block =
                    RevCodeBlock::new(file, name.clone(), index, line[..indent].to_string());
                /*if let Some(name) = &name {
                    let new_line = format!("{} {}", parser.comment_start(), name,);
                    block.push_line(new_line);
                }*/
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

        blocks
    }
}
