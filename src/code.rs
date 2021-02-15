use crate::config::{BlockLabels, ParserSettings};
use crate::util::Fallible;

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

pub fn parse(
    source: &str,
    parser: &ParserSettings,
    block_labels: &BlockLabels,
) -> Fallible<Vec<RevCodeBlock>> {
    let (start, next, end) = block_labels.label_prefixes();
    let block_name_sep = "#";

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

            let mut parts = full_name.splitn(3, block_name_sep);
            let file = parts.next().unwrap_or("").to_string();
            let name = parts.next().and_then(|s| {
                if s.is_empty() {
                    None
                } else {
                    Some(s.to_string())
                }
            });
            let index_str = parts
                .next()
                .ok_or_else(|| format!("Missing block index in {}", full_name))?;
            let index = index_str.parse::<usize>().map_err(|_| {
                format!(
                    "Can't parse block index '{}' to an integer in {}",
                    index_str, full_name
                )
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

            let block = RevCodeBlock::new(file, name.clone(), index, line[..indent].to_string());
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    #[test]
    fn no_block() {
        let config = toml::from_str::<Config>(include_str!("create/Yarner.toml")).unwrap();
        let labels = default_block_labels();

        let code = r#"
fn main() {}
"#;
        let blocks = parse(code, &config.parser, &labels).unwrap();

        assert_eq!(blocks.len(), 0);
    }

    #[test]
    fn simple_unnamed_block() {
        let config = toml::from_str::<Config>(include_str!("create/Yarner.toml")).unwrap();
        let labels = default_block_labels();

        let code = r#"
// <@README.md##0
fn main() {}
// @>README.md##0
"#;
        let blocks = parse(code, &config.parser, &labels).unwrap();

        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].name, None);
        assert_eq!(blocks[0].file, "README.md");
        assert_eq!(blocks[0].index, 0);
        assert_eq!(blocks[0].lines, vec!["fn main() {}"]);
    }

    #[test]
    fn simple_named_block() {
        let config = toml::from_str::<Config>(include_str!("create/Yarner.toml")).unwrap();
        let labels = default_block_labels();

        let code = r#"
// <@README.md#Block name#0
fn main() {}
// @>README.md#Block name#0
"#;
        let blocks = parse(code, &config.parser, &labels).unwrap();

        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].name, Some("Block name".to_owned()));
        assert_eq!(blocks[0].file, "README.md");
        assert_eq!(blocks[0].index, 0);
        assert_eq!(blocks[0].lines, vec!["fn main() {}"]);
    }

    #[test]
    fn nested_block() {
        let config = toml::from_str::<Config>(include_str!("create/Yarner.toml")).unwrap();
        let labels = default_block_labels();

        let code = r#"
// <@README.md##0
fn main() {}
// <@README.md#Inner#0
fn print() {}
// @>README.md#Inner#0
// @>README.md##0
"#;
        let blocks = parse(code, &config.parser, &labels).unwrap();

        assert_eq!(blocks.len(), 2);

        assert_eq!(blocks[0].name, Some("Inner".to_owned()));
        assert_eq!(blocks[0].file, "README.md");
        assert_eq!(blocks[0].index, 0);
        assert_eq!(blocks[0].lines, vec!["fn print() {}"]);

        assert_eq!(blocks[1].name, None);
        assert_eq!(blocks[1].file, "README.md");
        assert_eq!(blocks[1].index, 0);
        assert_eq!(blocks[1].lines, vec!["fn main() {}", "// ==> Inner."]);
    }

    #[test]
    fn multiple_block_same_name() {
        let config = toml::from_str::<Config>(include_str!("create/Yarner.toml")).unwrap();
        let labels = default_block_labels();

        let code = r#"
// <@README.md##0
fn main() {}
// <@README.md#Inner#0
fn print() {}
// <@>README.md#Inner#1
fn beep() {}
// @>README.md#Inner#1
// @>README.md##0
"#;
        let blocks = parse(code, &config.parser, &labels).unwrap();

        assert_eq!(blocks.len(), 3);

        assert_eq!(blocks[0].name, Some("Inner".to_owned()));
        assert_eq!(blocks[0].file, "README.md");
        assert_eq!(blocks[0].index, 0);
        assert_eq!(blocks[0].lines, vec!["fn print() {}"]);

        assert_eq!(blocks[1].name, Some("Inner".to_owned()));
        assert_eq!(blocks[1].file, "README.md");
        assert_eq!(blocks[1].index, 1);
        assert_eq!(blocks[1].lines, vec!["fn beep() {}"]);

        assert_eq!(blocks[2].name, None);
        assert_eq!(blocks[2].file, "README.md");
        assert_eq!(blocks[2].index, 0);
        assert_eq!(blocks[2].lines, vec!["fn main() {}", "// ==> Inner."]);
    }

    fn default_block_labels() -> BlockLabels {
        BlockLabels {
            comment_start: "//".to_string(),
            comment_end: None,
            block_start: "<@".to_string(),
            block_next: "<@>".to_string(),
            block_end: "@>".to_string(),
        }
    }
}
