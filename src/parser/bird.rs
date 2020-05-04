//! The parser for Bird Style literate programming.
//!
//! This includes some extensions to support some of the more advanced features of this tool.
//!
//! See `examples/bird/wc.c.lit` for an example of how to use this format with the default config, which is
//! specified as follows:
//!
//! *   Code lines begin with `> `, and must be preceded and followed by a blank line.
//! *   A macro (named code block) starts with `>>> Name of the code block` as its first line.
//! *   The comment symbol is `//`.
//! *   Interpolation of is done such as `@{a meta variable}`.
//! *   Macros (named code blocks) are invoked by `==> Macro name.` (note the period at the end)
//!
//! As with all supported styles, all code blocks with the same name are concatenated, in the order
//! they are found, and the "unnamed" block is used as the entry point when generating the output
//! source file. Any code blocks with names which are not invoked will not appear in the compiled
//! code.
//!
//! Currently, the Bird Style does not support code that is written to the compiled file, but not
//! rendered in the documentation file.

use serde_derive::{Deserialize, Serialize};
use std::iter::FromIterator;

use super::{ParseError, Parser, ParserConfig, Printer};

use crate::document::ast::Node;
use crate::document::code::CodeBlock;
use crate::document::text::TextBlock;
use crate::document::Document;
use crate::util::try_collect::TryCollectExt;

/// The config for parsing a Bird Style document
#[derive(Deserialize, Serialize, Debug)]
pub struct BirdParser {
    /// The line starter to identify code lines
    ///
    /// Default: `> `
    pub code_marker: String,
    /// The line starter to identify the name of a code block
    ///
    /// Default: `>>> `
    pub code_name_marker: String,
    /// The sequence to identify a comment which should be omitted from the compiled code.
    ///
    /// Default: `//`
    pub comment_start: String,
    /// The sequence to identify the start of a meta variable interpolation.
    ///
    /// Default: `@{`
    pub interpolation_start: String,
    /// The sequence to identify the end of a meta variable interpolation.
    ///
    /// Default: `}`
    pub interpolation_end: String,
    /// The sequence to identify the start of a macro invocation.
    ///
    /// Default: `==>`
    pub macro_start: String,
    /// The sequence to identify the end of a macro invocation.
    ///
    /// Default: `.`
    pub macro_end: String,
    /// The sequence to split variables into name and value.
    ///
    /// Default: `:`
    pub variable_sep: String,
    /// Prefix for file-specific entry points.
    ///
    /// Default: `file:`
    pub file_prefix: String,
    /// Name prefix for code blocks not shown in the docs.
    ///
    /// Default: `hidden:`
    pub hidden_prefix: String,
}

impl Default for BirdParser {
    fn default() -> Self {
        Self {
            code_marker: String::from("> "),
            code_name_marker: String::from(">>> "),
            comment_start: String::from("//"),
            interpolation_start: String::from("@{"),
            interpolation_end: String::from("}"),
            macro_start: String::from("==> "),
            macro_end: String::from("."),
            variable_sep: String::from(":"),
            file_prefix: String::from("file:"),
            hidden_prefix: String::from("hidden:"),
        }
    }
}

impl ParserConfig for BirdParser {
    fn comment_start(&self) -> &str {
        &self.comment_start
    }
    fn interpolation_start(&self) -> &str {
        &self.interpolation_start
    }
    fn interpolation_end(&self) -> &str {
        &self.interpolation_end
    }
    fn macro_start(&self) -> &str {
        &self.macro_start
    }
    fn macro_end(&self) -> &str {
        &self.macro_end
    }
    fn variable_sep(&self) -> &str {
        &self.variable_sep
    }
    fn file_prefix(&self) -> &str {
        &self.file_prefix
    }
    fn hidden_prefix(&self) -> &str {
        &self.hidden_prefix
    }
}

impl Parser for BirdParser {
    type Error = BirdError;

    fn parse<'a>(&self, input: &'a str) -> Result<Document<'a>, Self::Error> {
        struct State<'a> {
            node: Node<'a>,
            blank_line: bool,
        }

        enum Parse<'a> {
            Incomplete,
            Complete(Node<'a>),
            Error(BirdError),
        }

        let mut state = State {
            node: Node::Text(TextBlock::new()),
            blank_line: true,
        };
        let mut document = input
            .lines()
            .enumerate()
            .scan(&mut state, |state, (line_number, line)| match state {
                State { .. } if line.is_empty() => {
                    state.blank_line = true;
                    match &mut state.node {
                        Node::Code(..) => {
                            let mut new_block = TextBlock::new();
                            new_block.add_line(line);
                            let node = std::mem::replace(&mut state.node, Node::Text(new_block));
                            Some(Parse::Complete(node))
                        }
                        Node::Text(text_block) => {
                            text_block.add_line(line);
                            Some(Parse::Incomplete)
                        }
                    }
                }
                State {
                    blank_line: true, ..
                } if line.starts_with(&self.code_name_marker) => {
                    let (name, vars, defaults) =
                        match self.parse_name(&line[self.code_name_marker.len()..], false) {
                            Ok((name, vars, defaults)) => (name, vars, defaults),
                            Err(error) => {
                                return Some(Parse::Error(BirdError::Single {
                                    line_number,
                                    kind: error.into(),
                                }))
                            }
                        };
                    let hidden = name.starts_with(self.hidden_prefix());
                    let code_block = CodeBlock::new().named(name, vars, defaults).hidden(hidden);
                    state.blank_line = false;
                    let node = std::mem::replace(&mut state.node, Node::Code(code_block));
                    Some(Parse::Complete(node))
                }
                State {
                    blank_line: true, ..
                } if line.starts_with(&self.code_marker) => {
                    let line = match self.parse_line(line_number, &line[self.code_marker.len()..]) {
                        Ok(line) => line,
                        Err(error) => {
                            return Some(Parse::Error(BirdError::Single {
                                line_number,
                                kind: error.into(),
                            }))
                        }
                    };
                    let mut code_block = CodeBlock::new();
                    code_block.add_line(line);
                    let node = std::mem::replace(&mut state.node, Node::Code(code_block));
                    state.blank_line = false;
                    Some(Parse::Complete(node))
                }
                State {
                    node: Node::Code(code_block),
                    ..
                } if line.starts_with(&self.code_marker) => {
                    let line = match self.parse_line(line_number, &line[self.code_marker.len()..]) {
                        Ok(line) => line,
                        Err(error) => {
                            return Some(Parse::Error(BirdError::Single {
                                line_number,
                                kind: error.into(),
                            }))
                        }
                    };
                    code_block.add_line(line);
                    Some(Parse::Incomplete)
                }
                State {
                    node: Node::Code(..),
                    ..
                } => Some(Parse::Error(BirdError::Single {
                    line_number,
                    kind: BirdErrorKind::UnterminatedCodeBlock,
                })),
                State {
                    node: Node::Text(text_block),
                    ..
                } => {
                    state.blank_line = false;
                    text_block.add_line(&line);
                    Some(Parse::Incomplete)
                }
            })
            .filter_map(|parse| match parse {
                Parse::Incomplete => None,
                Parse::Error(error) => Some(Err(error)),
                Parse::Complete(node) => Some(Ok(node)),
            })
            .try_collect::<_, _, Vec<_>, BirdError>()?;
        document.push(state.node);
        Ok(Document::from_iter(document))
    }
}

impl Printer for BirdParser {
    fn print_text_block<'a>(&self, block: &TextBlock<'a>) -> String {
        format!("{}\n", block.to_string())
    }

    fn print_code_block<'a>(&self, block: &CodeBlock<'a>) -> String {
        let mut output = String::new();
        if let Some(name) = &block.name {
            output.push_str(&self.code_name_marker);
            output.push_str(&self.print_name(name.clone(), &block.vars, &block.defaults));
            output.push('\n');
        }
        for line in &block.source {
            output.push_str(&self.code_marker);
            output.push_str(&self.print_line(&line, true));
            output.push('\n');
        }
        output
    }
}

/// Kinds of errors that can be encountered while parsing and restructuring the document
#[derive(Debug)]
pub enum BirdErrorKind {
    /// The code block has ended, but there is no blank line
    UnterminatedCodeBlock,
    /// Generic parse error
    Parse(ParseError),
}

/// Errors that were encountered while parsing the document
#[derive(Debug)]
pub enum BirdError {
    #[doc(hidden)]
    Single {
        line_number: usize,
        kind: BirdErrorKind,
    },
    #[doc(hidden)]
    Multi(Vec<BirdError>),
}

impl std::fmt::Display for BirdError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BirdError::Multi(errors) => {
                for error in errors {
                    writeln!(f, "{}", error)?;
                }
                Ok(())
            }
            BirdError::Single { line_number, kind } => {
                writeln!(f, "{:?} (line {})", kind, line_number)
            }
        }
    }
}

impl std::error::Error for BirdError {}

impl FromIterator<BirdError> for BirdError {
    fn from_iter<I: IntoIterator<Item = BirdError>>(iter: I) -> Self {
        BirdError::Multi(iter.into_iter().collect())
    }
}

impl From<Vec<BirdError>> for BirdError {
    fn from(multi: Vec<BirdError>) -> Self {
        BirdError::Multi(multi)
    }
}

impl From<ParseError> for BirdErrorKind {
    fn from(error: ParseError) -> Self {
        BirdErrorKind::Parse(error)
    }
}
