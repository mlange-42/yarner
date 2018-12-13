//! The parser for Bird Style literate programming.
//!
//! This includes some extensions to support some of the more advanced features of this tool.
//!
//! See `wc.c.lit` for an example of how to use this format with the default config, which is
//! specified as follows:
//!
//! *   Code lines begin with `> `, and must be preceded and followed by a blank line.
//! *   A named code block starts with `>>> Name of the code block` as its first line.
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

use std::iter::FromIterator;
use serde_derive::Deserialize;

use super::{Printer, Parser, ParserConfig, ParseError};

use crate::document::Document;
use crate::document::ast::Node;
use crate::document::code::CodeBlock;
use crate::document::text::TextBlock;
use crate::util::try_collect::TryCollectExt;

#[derive(Deserialize, Debug)]
pub struct BirdParser {
    code_marker: String,
    code_name_marker: String,
    comment_start: String,
    interpolation_start: String,
    interpolation_end: String,
    macro_start: String,
    macro_end: String,
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
        }
    }
}

impl ParserConfig for BirdParser {
    fn comment_start(&self) -> &str { &self.comment_start }
    fn interpolation_start(&self) -> &str { &self.interpolation_start }
    fn interpolation_end(&self) -> &str { &self.interpolation_end }
    fn macro_start(&self) -> &str { &self.macro_start }
    fn macro_end(&self) -> &str { &self.macro_end }
}

impl Parser for BirdParser {
    type Config = BirdParser;
    type Error = BirdError;
    fn config(&self) -> &Self::Config { self }

    fn parse<'a>(&self, input: &'a str) -> Result<Document<'a>, Self::Error> {
        #[derive(Default)]
        struct State<'a> {
            node: Option<Node<'a>>,
        }

        enum Parse<'a> {
            Incomplete,
            Complete(Node<'a>),
            Error(BirdError),
        }

        let mut state = State::default();
        let mut document = input.lines()
            .enumerate()
            .scan(&mut state, |state, (line_number, line)| match state.node {
                None if line.is_empty() => Some(Parse::Complete(Node::Text(TextBlock::new()))),
                None if line.starts_with(&self.code_name_marker) => {
                    let (name, vars) = match self.parse_name(&line[self.code_name_marker.len()..]) {
                        Ok((name, vars)) => (name, vars),
                        Err(error) => return Some(Parse::Error(BirdError::Single { line_number, kind: error.into() })),
                    };
                    let code_block = CodeBlock::new().named(name, vars);
                    state.node = Some(Node::Code(code_block));
                    Some(Parse::Incomplete)
                }
                None if line.starts_with(&self.code_marker) => {
                    let line = match self.parse_line(line_number, &line[self.code_marker.len()..]) {
                        Ok(line) => line,
                        Err(error) => return Some(Parse::Error(BirdError::Single { line_number, kind: error.into() })),
                    };
                    let code_block = CodeBlock::new().add_line(line);
                    state.node = Some(Node::Code(code_block));
                    Some(Parse::Incomplete)
                }
                None => {
                    let text_block = TextBlock::new().add_line(&line);
                    state.node = Some(Node::Text(text_block));
                    Some(Parse::Incomplete)
                }
                Some(..) if line.is_empty() => {
                    let node = state.node.take().unwrap();
                    Some(Parse::Complete(node))
                }
                Some(Node::Code(..)) if line.starts_with(&self.code_marker) => {
                    if let Node::Code(code_block) = state.node.take().unwrap() {
                        let line = match self.parse_line(line_number, &line[self.code_marker.len()..]) {
                            Ok(line) => line,
                            Err(error) => return Some(Parse::Error(BirdError::Single { line_number, kind: error.into() })),
                        };
                        let code_block = code_block.add_line(line);
                        state.node = Some(Node::Code(code_block));
                    } else { unreachable!(); }
                    Some(Parse::Incomplete)
                }
                Some(Node::Code(..)) => {
                    Some(Parse::Error(BirdError::Single { line_number, kind: BirdErrorKind::UnterminatedCodeBlock }))
                }
                Some(Node::Text(..)) => {
                    if let Node::Text(text_block) = state.node.take().unwrap() {
                        state.node = Some(Node::Text(text_block.add_line(line)));
                    } else { unreachable!(); }
                    Some(Parse::Incomplete)
                }
            })
            .filter_map(|parse| match parse {
                Parse::Incomplete => None,
                Parse::Error(error) => Some(Err(error)),
                Parse::Complete(node) => Some(Ok(node)),
            })
            .try_collect::<_, _, Vec<_>, BirdError>()?;
        if let Some(node) = state.node.take() {
            document.push(node);
        }
        Ok(Document::from_iter(document))
    }
}

impl Printer for BirdParser {
    type Config = BirdParser;
    fn config(&self) -> &Self::Config { self }

    fn print_text_block<'a>(&self, block: &TextBlock<'a>) -> String {
        format!("{}\n", block.to_string())
    }

    fn print_code_block<'a>(&self, block: &CodeBlock<'a>) -> String {
        let mut output = String::new();
        if let Some(name) = &block.name {
            let var_placeholder = format!("{}{}", self.interpolation_start, self.interpolation_end);
            let mut name = name.clone();
            for var in &block.vars {
                let var_name = format!("{}{}{}", self.interpolation_start, var, self.interpolation_end);
                name = name.replacen(&var_placeholder, &var_name, 1);
            }
            output.push_str(&format!("{}{}\n", self.code_name_marker, name));
        }
        for line in &block.source {
            output.push_str(&format!("{}{}\n", self.code_marker, self.print_line(&line)));
        }
        output
    }
}

#[derive(Debug)]
pub enum BirdErrorKind {
    UnterminatedCodeBlock,
    Parse(ParseError),
}

#[derive(Debug)]
pub enum BirdError {
    Single {
        line_number: usize,
        kind: BirdErrorKind,
    },
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
            BirdError::Single { line_number, kind } => writeln!(f, "{:?} (line {})", kind, line_number),
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
