//! The built in parsers
//!
//! Additional parsers should implement the [`Parser`] trait, as well as define a corresponding
//! [`ParserConfig`] type to allow configuring that parser

pub mod bird;
pub mod latex;
pub mod markdown;
pub mod html;

pub use self::bird::BirdParser;

use crate::document::Document;
use crate::document::code::{Line, CodeBlock, Source, Segment};
use crate::document::text::TextBlock;

/// A [`ParserConfig`] can be used to customize the built in parsing methods
pub trait ParserConfig {
    /// The token to denote the start of a comment that should be rendered in the documentation.
    /// This may be specified as the actual line comment symbol used by the source language to
    /// exclude all comments from the transpiled file, or it can be a completely separate symbol
    /// to allow for some comments to be left untouched even in the output files.
    fn comment_start(&self) -> &str;
    /// The token to denote the start of a meta-variable interpolation
    fn interpolation_start(&self) -> &str;
    /// The token to denote the end of a meta-variable interpolation
    fn interpolation_end(&self) -> &str;
    /// The token to denote the start of a macro invocation
    fn macro_start(&self) -> &str;
    /// The token to denote the end of a macro invocation
    fn macro_end(&self) -> &str;
}

/// A [`Parser`] determines which lines are code and which are text, and may use its `Config` to
/// actually handle reading the lines of code
pub trait Parser {
    type Config: ParserConfig;
    type Error: std::error::Error;

    /// The config object to use in the code parser
    fn config(&self) -> &Self::Config;

    /// Parses the text part of the document. Should delegate the code section on a line-by-line
    /// basis to the built in code parser.
    fn parse<'a>(&self, input: &'a str) -> Result<Document<'a>, Self::Error>;

    /// Parses a macro name, returning the name and the extracted variables
    fn parse_name<'a>(&self, mut input: &'a str) -> Result<(String, Vec<&'a str>), ParseError> {
        let mut name = String::new();
        let mut vars = vec![];
        let config = self.config();
        let start = config.interpolation_start();
        let end = config.interpolation_end();
        loop {
            if let Some(start_index) = input.find(start) {
                if let Some(end_index) = input[start_index + start.len()..].find(end) {
                    name.push_str(&input[..start_index]);
                    vars.push(&input[start_index + start.len()..start_index + start.len() + end_index]);
                    input = &input[start_index + start.len() + end_index + end.len()..];
                }
            } else {
                name.push_str(input);
                break;
            }
        }
        return Ok((name, vars));
    }

    /// Parses a line as code, returning the parsed [`Line`] object
    fn parse_line<'a>(&self, line_number: usize, input: &'a str) -> Result<Line<'a>, ParseError> {
        let config = self.config();

        let indent_len = input.chars()
            .take_while(|ch| ch.is_whitespace())
            .collect::<String>()
            .len();
        let (indent, rest) = input.split_at(indent_len);
        let (mut rest, comment) = if let Some(comment_index) = rest.find(config.comment_start()) {
            let (rest, comment) = rest.split_at(comment_index);
            (rest, Some(comment))
        } else {
            (rest, None)
        };

        if rest.starts_with(config.macro_start()) {
            if let Some(end_index) = rest.find(config.macro_end()) {
                let (name, scope) = self.parse_name(&rest[config.macro_start().len()..end_index])?;
                return Ok(Line {
                    line_number,
                    indent,
                    source: Source::Macro { name, scope },
                    comment
                });
            }
        }

        let mut source = vec![];
        let start = config.interpolation_start();
        let end = config.interpolation_end();
        loop {
            if let Some(start_index) = rest.find(start) {
                if let Some(end_index) = rest[start_index + start.len()..].find(end) {
                    source.push(Segment::Source(&rest[..start_index]));
                    source.push(Segment::MetaVar(&rest[start_index + start.len()..start_index + start.len() + end_index]));
                    rest = &rest[start_index + start.len() + end_index + end.len()..];
                }
            } else {
                if !rest.is_empty() {
                    source.push(Segment::Source(rest));
                }
                break;
            }
        }

        Ok(Line {
            line_number,
            indent,
            source: Source::Source(source),
            comment,
        })
    }
}

#[derive(Debug)]
pub enum ParseError {} // is there even such a thing as a parse error? who knows.

/// A [`Printer`] can invert the parsing process, printing the code blocks how they should be
/// rendered in the documentation text.
pub trait Printer {
    type Config: ParserConfig;

    /// The config object to use in the code parser
    fn config(&self) -> &Self::Config;

    /// Prints a code block
    fn print_code_block<'a>(&self, block: &CodeBlock<'a>) -> String;

    /// Prints a text block
    fn print_text_block<'a>(&self, block: &TextBlock<'a>) -> String;

    /// Prints a line of a code block
    fn print_line<'a>(&self, line: &Line<'a>) -> String {
        let mut output = format!("{}", line.indent);
        let config = self.config();
        match &line.source {
            Source::Macro { name, scope } => {
                let var_placeholder = format!("{}{}", config.interpolation_start(), config.interpolation_end());
                let mut name = name.clone();
                for var in scope {
                    let var_name = format!("{}{}{}", config.interpolation_start(), var, config.interpolation_end());
                    name = name.replacen(&var_placeholder, &var_name, 1);
                }
                output.push_str(&format!("{}{}{}", config.macro_start(), name, config.macro_end()));
            },
            Source::Source(segments) => {
                for segment in segments {
                    match segment {
                        Segment::Source(source) => output.push_str(source),
                        Segment::MetaVar(name) => output.push_str(&format!("{}{}{}", config.interpolation_start(), name, config.interpolation_end())),
                    }
                }
            }
        }
        if let Some(comment) = line.comment {
            output.push_str(config.comment_start());
            output.push_str(comment);
        }
        output
    }
}
