//! The parser for TeX based literate programming.
//!
//! See `examples/tex/wc.c.tex` for an example of how to use this format with the default config,
//! which is specified as follows:
//!
//! *   Code is written in the `code` environment.
//! *   A code block can be named by passing the name as a square bracket argument
//!     `\begin{code}[name=Macro name]`
//! *   The language of a code block can be specified similarly: `\begin{code}[language=c]`
//! *   The comment symbol is `//`, but they are rendered inline
//! *   Interpolation of is done such as `@{a meta variable}`.
//! *   Macros (named code blocks) are invoked by `==> Macro name.` (note the period at the end)
//!
//! As with all supported styles, all code blocks with the same name are concatenated, in the order
//! they are found, and the "unnamed" block is used as the entry point when generating the output
//! source file. Any code blocks with names which are not invoked will not appear in the compiled
//! code.
//!
//! Percent signs in the *code blocks* do not need to be escaped (they won't be interpreted as
//! TeX comments).
//!
//! Note that due to the "stupid" parsing, the entire `\begin{code}[language=c,name=Macro name]`
//! must be written all on one line. The order of language and name, if both are specified, is not
//! important. Complex names may be grouped with `{}`.
//!
//! Currently, the TeX parser does not support code that is written to the compiled file, but
//! not rendered in the documentation file.

use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use std::iter::FromIterator;

use super::{ParseError, Parser, ParserConfig, Printer};

use crate::document::ast::Node;
use crate::document::code::CodeBlock;
use crate::document::text::TextBlock;
use crate::document::tranclusion::Transclusion;
use crate::document::Document;
use crate::util::try_collect::TryCollectExt;
use std::path::PathBuf;

/// The config for parsing a TeX document
#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct TexParser {
    /// The environment used to indicate code.
    ///
    /// Default: `code`
    pub code_environment: String,
    /// The language to set if there was no automatically detected language. Optional
    pub default_language: Option<String>,
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

impl Default for TexParser {
    fn default() -> Self {
        Self {
            default_language: None,
            code_environment: String::from("code"),
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

impl TexParser {
    /// Creates a default parser with a fallback language
    pub fn for_language(language: String) -> Self {
        Self {
            default_language: Some(language),
            ..Self::default()
        }
    }

    /// Sets the default language of this parser (or does nothing if `None` is passed)
    pub fn default_language(&self, language: Option<String>) -> Self {
        if let Some(language) = language {
            Self {
                default_language: Some(language),
                ..self.clone()
            }
        } else {
            self.clone()
        }
    }
}

impl ParserConfig for TexParser {
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
}

impl TexParser {
    fn parse_arguments<'a>(
        &self,
        arg_string: &'a str,
    ) -> Result<Option<HashMap<&'a str, &'a str>>, TexErrorKind> {
        if arg_string.starts_with('[') {
            if !arg_string.ends_with(']') {
                return Err(TexErrorKind::UnclosedArgumentList);
            }
            let mut args = HashMap::new();
            let mut arg_string = &arg_string[1..arg_string.len() - 1];
            while !arg_string.is_empty() {
                let equal = match arg_string.find('=') {
                    Some(index) => index,
                    None => return Err(TexErrorKind::MissingValueForArgument),
                };
                let (name, rest) = arg_string.split_at(equal);
                let rest = &rest[1..];
                let (value, rest) = if rest.starts_with('{') {
                    let rest = &rest[1..];
                    let value_len = rest
                        .chars()
                        .scan((1, '{'), |state, ch| {
                            let previous = std::mem::replace(&mut state.1, ch);
                            if previous == '\\' {
                                return Some(ch);
                            } else if ch == '{' {
                                state.0 += 1;
                            } else if ch == '}' {
                                state.0 -= 1;
                            }
                            if state.0 == 0 {
                                None
                            } else {
                                Some(ch)
                            }
                        })
                        .collect::<String>()
                        .len();
                    let (value, rest) = rest[..value_len].split_at(value_len);
                    if !rest.is_empty() && !rest[1..].starts_with(',') {
                        return Err(TexErrorKind::InvalidArgumentList);
                    }
                    (value, if rest.is_empty() { rest } else { &rest[1..] })
                } else {
                    let comma = match rest.find(',') {
                        Some(index) => index,
                        None => rest.len(),
                    };
                    let (value, rest) = rest.split_at(comma);
                    (value, if rest.is_empty() { rest } else { &rest[1..] })
                };
                args.insert(name.trim(), value.trim());
                arg_string = rest;
            }
            Ok(Some(args))
        } else {
            Ok(None)
        }
    }
}

impl Parser for TexParser {
    type Error = TexError;

    fn parse(&self, input: &str) -> Result<Document, Self::Error> {
        struct State {
            node: Node,
        }

        enum Parse {
            Incomplete,
            Complete(Node),
            Error(TexError),
        }

        let env_start = format!("\\begin{{{}}}", self.code_environment);
        let env_end = format!("\\end{{{}}}", self.code_environment);

        let mut state = State {
            node: Node::Text(TextBlock::new()),
        };

        let mut document = input
            .lines()
            .enumerate()
            .scan(&mut state, |state, (line_number, line)| {
                match &mut state.node {
                    Node::Code(code_block) => {
                        if !line.starts_with(&code_block.indent) {
                            return Some(Parse::Error(TexError::Single {
                                line_number,
                                kind: TexErrorKind::IncorrectIndentation,
                            }));
                        }
                        let line = &line[code_block.indent.len()..];
                        if line.starts_with(&env_end) {
                            let node =
                                std::mem::replace(&mut state.node, Node::Text(TextBlock::new()));
                            Some(Parse::Complete(node))
                        } else {
                            let line = match self.parse_line(line_number, line) {
                                Ok(line) => line,
                                Err(error) => {
                                    return Some(Parse::Error(TexError::Single {
                                        line_number,
                                        kind: error.into(),
                                    }))
                                }
                            };
                            code_block.add_line(line);
                            Some(Parse::Incomplete)
                        }
                    }
                    Node::Text(text_block) => {
                        if line.trim_start().starts_with(&env_start) {
                            let indent_length = line.find(&env_start).unwrap();
                            let (indent, rest) = line.split_at(indent_length);
                            let rest = &rest[env_start.len()..].trim();
                            let args = match self.parse_arguments(rest) {
                                Ok(args) => args,
                                Err(kind) => {
                                    return Some(Parse::Error(TexError::Single {
                                        line_number,
                                        kind,
                                    }))
                                }
                            };
                            let mut code_block = CodeBlock::new().indented(indent);
                            if let Some(args) = args {
                                if let Some(name) = args.get("name") {
                                    let (name, vars, defaults) = match self.parse_name(name, false)
                                    {
                                        Ok(name) => name,
                                        Err(error) => {
                                            return Some(Parse::Error(TexError::Single {
                                                line_number,
                                                kind: error.into(),
                                            }))
                                        }
                                    };
                                    let hidden = name.starts_with(&self.hidden_prefix);
                                    code_block =
                                        code_block.named(name, vars, defaults).hidden(hidden);
                                }
                                code_block = match args.get("language") {
                                    Some(language) => code_block.in_language(language.to_string()),
                                    None => match &self.default_language {
                                        Some(language) => {
                                            code_block.in_language(language.to_string())
                                        }
                                        None => code_block,
                                    },
                                };
                            }
                            let node = std::mem::replace(&mut state.node, Node::Code(code_block));
                            Some(Parse::Complete(node))
                        } else {
                            text_block.add_line(line);
                            Some(Parse::Incomplete)
                        }
                    }
                    Node::Transclusion(_) => Some(Parse::Incomplete), // TODO?
                }
            })
            .filter_map(|parse| match parse {
                Parse::Incomplete => None,
                Parse::Error(error) => Some(Err(error)),
                Parse::Complete(node) => Some(Ok(node)),
            })
            .try_collect::<_, _, Vec<_>, TexError>()?;
        document.push(state.node);
        Ok(Document::from_iter(document))
    }

    fn find_links(&self, _input: &Document) -> Result<Vec<PathBuf>, Self::Error> {
        Ok(vec![])
    }
}

impl Printer for TexParser {
    fn print_text_block(&self, block: &TextBlock) -> String {
        format!("{}\n", block.to_string())
    }

    fn print_code_block(&self, block: &CodeBlock) -> String {
        let mut output = format!("\\begin{{{}}}", self.code_environment);
        if block.language.is_some() || block.name.is_some() {
            output.push('[');
        }
        if let Some(language) = &block.language {
            output.push_str("language=");
            output.push_str(language);
            output.push(',');
        }
        if let Some(name) = &block.name {
            output.push_str("name={");
            output.push_str(
                &name
                    .replace("{", "\\{")
                    .replace("}", "\\}")
                    .replace("_", "\\_"),
            );
            output.push('}');
        }
        if block.language.is_some() || block.name.is_some() {
            output.push(']');
        }
        output.push('\n');

        for line in &block.source {
            output.push_str(&self.print_line(&line, true));
            output.push('\n');
        }
        output.push_str(&format!("\\end{{{}}}\n", self.code_environment));

        output
    }

    fn print_transclusion(&self, _transclusion: &Transclusion) -> String {
        // TODO
        String::new()
    }
}

/// Kinds of errors that can be encountered while parsing and restructuring the TeX document
#[derive(Debug)]
pub enum TexErrorKind {
    /// A line was un-indented too far, usually indicating an error
    IncorrectIndentation,
    /// An argument was passed with no value
    MissingValueForArgument,
    /// The closing `]` is missing from the argument lisst
    UnclosedArgumentList,
    /// There is a syntax error in the argument list
    InvalidArgumentList,
    /// Generic parse error
    Parse(ParseError),
}

/// Errors that were encountered while parsing the TeX document
#[derive(Debug)]
pub enum TexError {
    #[doc(hidden)]
    Single {
        line_number: usize,
        kind: TexErrorKind,
    },
    #[doc(hidden)]
    Multi(Vec<TexError>),
}

impl std::fmt::Display for TexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TexError::Multi(errors) => {
                for error in errors {
                    writeln!(f, "{}", error)?;
                }
                Ok(())
            }
            TexError::Single { line_number, kind } => {
                writeln!(f, "{:?} (line {})", kind, line_number)
            }
        }
    }
}

impl std::error::Error for TexError {}

impl FromIterator<TexError> for TexError {
    fn from_iter<I: IntoIterator<Item = TexError>>(iter: I) -> Self {
        TexError::Multi(iter.into_iter().collect())
    }
}

impl From<Vec<TexError>> for TexError {
    fn from(multi: Vec<TexError>) -> Self {
        TexError::Multi(multi)
    }
}

impl From<ParseError> for TexErrorKind {
    fn from(error: ParseError) -> Self {
        TexErrorKind::Parse(error)
    }
}
