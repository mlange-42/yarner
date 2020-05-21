//! The built in parsers
//!
//! Additional parsers should implement the `Parser` trait, as well as define a corresponding
//! `ParserConfig` type to allow configuring that parser.
//!
//! Additionally, for each parser, a `Printer` is needed to be able to write the code back
//! out correctly.

pub mod html;
pub mod md;
pub mod tex;

pub use self::html::HtmlParser;
pub use self::md::MdParser;
pub use self::tex::TexParser;

use crate::document::code::{CodeBlock, Line, Segment, Source};
use crate::document::text::TextBlock;
use crate::document::tranclusion::Transclusion;
use crate::document::Document;
use std::error::Error;
use std::fmt;
use std::path::PathBuf;

/// A `ParserConfig` can be used to customize the built in parsing methods
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
    /// The sequence to split variables into name and value.
    fn variable_sep(&self) -> &str;
    /// Prefix for file-specific entry points.
    fn file_prefix(&self) -> &str;
}

/// A `Parser` determines which lines are code and which are text, and may use its `Config` to
/// actually handle reading the lines of code
pub trait Parser: ParserConfig {
    /// The type of error for this parser
    type Error: std::error::Error;

    /// Parses the text part of the document. Should delegate the code section on a line-by-line
    /// basis to the built in code parser.
    fn parse(&self, input: &str) -> Result<Document, Self::Error>;

    /// Find all files linked into the document for later compilation and/or transclusion.
    fn find_links(&self, input: &Document, from: &PathBuf) -> Result<Vec<PathBuf>, Self::Error>;

    /// Parses a macro name, returning the name and the extracted variables
    #[allow(clippy::type_complexity)]
    fn parse_name(
        &self,
        mut input: &str,
        is_call: bool,
    ) -> Result<(String, Vec<String>, Vec<Option<String>>), ParseError> {
        let orig = input;
        let mut name = String::new();
        let mut vars = vec![];
        let mut optionals = vec![];
        let start = self.interpolation_start();
        let end = self.interpolation_end();
        let sep = self.variable_sep();
        let sep_len = sep.len();
        loop {
            if let Some(start_index) = input.find(start) {
                if let Some(end_index) = input[start_index + start.len()..].find(end) {
                    name.push_str(&input[..start_index]);
                    name.push_str(&start);
                    name.push_str(&end);
                    let var =
                        &input[start_index + start.len()..start_index + start.len() + end_index];
                    if is_call {
                        vars.push(var.to_owned());
                        optionals.push(None);
                    } else if let Some(sep_index) = var.find(sep) {
                        vars.push((&var[..sep_index]).to_owned());
                        optionals.push(Some((&var[sep_index + sep_len..]).to_owned()));
                    } else {
                        vars.push(var.to_owned());
                        optionals.push(None);
                    }
                    input = &input[start_index + start.len() + end_index + end.len()..];
                } else {
                    return Err(ParseError::UnclosedVariableError(orig.to_owned()));
                }
            } else {
                name.push_str(input);
                break;
            }
        }
        Ok((name, vars, optionals))
    }

    /// Parses a line as code, returning the parsed `Line` object
    fn parse_line(&self, line_number: usize, input: &str) -> Result<Line, ParseError> {
        let orig = input;
        let indent_len = input
            .chars()
            .take_while(|ch| ch.is_whitespace())
            .collect::<String>()
            .len();
        let (indent, rest) = input.split_at(indent_len);
        let (mut rest, comment) = if let Some(comment_index) = rest.find(self.comment_start()) {
            let (rest, comment) = rest.split_at(comment_index);
            (
                rest,
                Some((&comment[self.comment_start().len()..]).to_owned()),
            )
        } else {
            (rest, None)
        };

        if rest.starts_with(self.macro_start()) {
            if let Some(end_index) = rest.find(self.macro_end()) {
                let (name, scope, _names) =
                    self.parse_name(&rest[self.macro_start().len()..end_index], true)?;
                return Ok(Line {
                    line_number,
                    indent: indent.to_owned(),
                    source: Source::Macro { name, scope },
                    comment,
                });
            }
        }

        let mut source = vec![];
        let start = self.interpolation_start();
        let end = self.interpolation_end();
        loop {
            if let Some(start_index) = rest.find(start) {
                if let Some(end_index) = rest[start_index + start.len()..].find(end) {
                    source.push(Segment::Source((&rest[..start_index]).to_owned()));
                    source.push(Segment::MetaVar(
                        (&rest[start_index + start.len()..start_index + start.len() + end_index])
                            .to_owned(),
                    ));
                    rest = &rest[start_index + start.len() + end_index + end.len()..];
                } else {
                    return Err(ParseError::UnclosedVariableError(orig.to_owned()));
                }
            } else {
                if !rest.is_empty() {
                    source.push(Segment::Source(rest.to_owned()));
                }
                break;
            }
        }

        Ok(Line {
            line_number,
            indent: indent.to_owned(),
            source: Source::Source(source),
            comment,
        })
    }

    /// Finds all file-specific entry points
    fn get_entry_points(&self, doc: &Document, language: Option<&str>) -> Vec<(String, String)> {
        let mut entries = vec![];
        let pref = self.file_prefix();
        for (name, _block) in doc.tree().code_blocks(language) {
            if let Some(name) = name {
                if name.starts_with(pref) {
                    entries.push((name.to_owned(), (&name[pref.len()..]).to_owned()))
                }
            }
        }
        entries
    }
}

/// A generic parse error
#[derive(Debug)]
pub enum ParseError {
    /// Error for unclosed variables, e.g. @{ without }
    UnclosedVariableError(String),
    /// Error for unclosed transclusions, e.g. @{{ without }}
    UnclosedTransclusionError(String),
    /// Error for invalid transclusions, e.g. if the file is not found
    InvalidTransclusionError(String),
    /// Error for multiple locations with entrypoints to the same code file
    MultipleCodeFileAccessError(String),
} // is there even such a thing as a parse error? who knows.

impl ParseError {
    fn message(&self) -> &str {
        match self {
            ParseError::UnclosedVariableError(s) => s,
            ParseError::UnclosedTransclusionError(s) => s,
            ParseError::InvalidTransclusionError(s) => s,
            ParseError::MultipleCodeFileAccessError(s) => s,
        }
    }
}

impl Error for ParseError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}
impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message())
    }
}

/// A `Printer` can invert the parsing process, printing the code blocks how they should be
/// rendered in the documentation text.
pub trait Printer: ParserConfig {
    /// Prints a code block
    fn print_code_block(&self, block: &CodeBlock) -> String;

    /// Prints a text block
    fn print_text_block(&self, block: &TextBlock) -> String;

    /// Prints a code block
    fn print_transclusion(&self, transclusion: &Transclusion) -> String;

    /// Fills a name with its placeholders and defaults
    fn print_name(&self, mut name: String, vars: &[String], defaults: &[Option<String>]) -> String {
        let start = self.interpolation_start();
        let end = self.interpolation_end();
        let var_placeholder = format!("{}{}", start, end);
        for (var, default) in vars.iter().zip(defaults) {
            let mut var_full = var.to_string();
            if let Some(default) = default {
                var_full.push_str(self.variable_sep());
                var_full.push_str(default);
            }
            let var_name = format!("{}{}{}", start, var_full, end);
            name = name.replacen(&var_placeholder, &var_name, 1);
        }
        name
    }

    /// Fills a name with its placeholders
    fn print_macro_call(&self, mut name: String, vars: &[String]) -> String {
        let start = self.interpolation_start();
        let end = self.interpolation_end();
        let var_placeholder = format!("{}{}", start, end);
        for var in vars {
            let var_name = format!("{}{}{}", start, var, end);
            name = name.replacen(&var_placeholder, &var_name, 1);
        }
        name
    }

    /// Prints a line of a code block
    fn print_line(&self, line: &Line, print_comments: bool) -> String {
        let mut output = line.indent.to_string();
        match &line.source {
            Source::Macro { name, scope } => {
                output.push_str(self.macro_start());
                output.push_str(&self.print_macro_call(name.clone(), &scope));
                output.push_str(self.macro_end());
            }
            Source::Source(segments) => {
                for segment in segments {
                    match segment {
                        Segment::Source(source) => output.push_str(source),
                        Segment::MetaVar(name) => {
                            output.push_str(self.interpolation_start());
                            output.push_str(&name);
                            output.push_str(self.interpolation_end());
                        }
                    }
                }
            }
        }
        if print_comments {
            if let Some(comment) = &line.comment {
                output.push_str(self.comment_start());
                output.push_str(&comment);
            }
        }
        output
    }
}
