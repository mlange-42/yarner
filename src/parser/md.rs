//! The parser for Markdown based literate programming.
//!
//! This includes some extensions to support some of the more advanced features of this tool.
//!
//! See `examples/md/wc.c.md` for an example of how to use this format with the default config,
//! which is specified as follows:
//!
//! *   Code lines are enclosed in fenced code blocks, using `\`\`\`lang` as the fence.
//! *   A macro (named code block) separates the name from the language tag ` - `, such as
//!     `\`\`\`c - Name of the macro`. Note that the hyphen is surrounded by a single space on
//!     either side, even if there is no language tag.
//! *   The comment symbol is `//`, but they are rendered inline
//!     *   Enabling the `comments_as_aside` option will render comments in an `<aside>` after the
//!         code block, which you may then style accordingly when the document is rendered.
//! *   Interpolation of is done such as `@{a meta variable}`.
//! *   Macros (named code blocks) are invoked by `==> Macro name.` (note the period at the end)
//!
//! As with all supported styles, all code blocks with the same name are concatenated, in the order
//! they are found, and the "unnamed" block is used as the entry point when generating the output
//! source file. Any code blocks with names which are not invoked will not appear in the compiled
//! code.
//!
//! Currently, the Markdown parser does not support code that is written to the compiled file, but
//! not rendered in the documentation file.

use serde_derive::{Deserialize, Serialize};
use std::iter::FromIterator;

use super::{ParseError, Parser, ParserConfig, Printer};

use crate::document::ast::Node;
use crate::document::code::CodeBlock;
use crate::document::text::TextBlock;
use crate::document::tranclusion::Transclusion;
use crate::document::Document;
use crate::util::try_collect::TryCollectExt;
use regex::Regex;
use std::fs::File;
use std::path::PathBuf;

/// The config for parsing a Markdown document
#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct MdParser {
    /// The sequence that identifies the start and end of a fenced code block
    ///
    /// Default: `\`\`\``
    pub fence_sequence: String,
    /// Alternative sequence that identifies the start and end of a fenced code block.
    /// Allows for normal Markdown fences in code blocks
    ///
    /// Default: \~\~\~
    pub fence_sequence_alt: String,
    /// The sequence that separates the language from the name of the code block after the fence
    ///
    /// Default: ` - `
    pub block_name_start: String,
    /// The sequence that indicates the end of the code block name. Optional
    pub block_name_end: Option<String>,
    /// Parsed comments are stripped from the code and written to an `<aside></aside>` block after
    /// the code when printing. If false, the comments are just written back into the code.
    ///
    /// Default: `false`
    pub comments_as_aside: bool,
    /// The language to set if there was no automatically detected language. Optional
    pub default_language: Option<String>,
    /// The sequence to identify a comment which should be omitted from the compiled code, and may
    /// be rendered as an `<aside>` if `comments_as_aside` is set.
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
    /// The sequence to identify the start of a transclusion.
    ///
    /// Default: `@{{`
    pub transclusion_start: String,
    /// The sequence to identify the end of a transclusion.
    ///
    /// Default: `}}`
    pub transclusion_end: String,
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
    /// Determines if code lines containing only whitespace characters are printed as blank lines.
    ///
    /// Default: true
    pub blank_lines: bool,
}

impl Default for MdParser {
    fn default() -> Self {
        Self {
            default_language: None,
            fence_sequence: String::from("```"),
            fence_sequence_alt: String::from("~~~"),
            block_name_start: String::from(" - "),
            block_name_end: None,
            comments_as_aside: false,
            comment_start: String::from("//"),
            interpolation_start: String::from("@{"),
            interpolation_end: String::from("}"),
            macro_start: String::from("==> "),
            macro_end: String::from("."),
            transclusion_start: String::from("@{{"),
            transclusion_end: String::from("}}"),
            variable_sep: String::from(":"),
            file_prefix: String::from("file:"),
            hidden_prefix: String::from("hidden:"),
            blank_lines: true,
        }
    }
}

impl MdParser {
    const LINK_PATTERN: &'static str = r"\[([^\[\]]*)\]\((.*?)\)";

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

    fn parse_transclusion(&self, line: &str) -> Result<Option<Node>, ParseError> {
        let trim = line.trim();
        if trim.starts_with(&self.transclusion_start) {
            if let Some(index) = line.find(&self.transclusion_end) {
                let trans = &trim[..index];
                let regex = Regex::new(Self::LINK_PATTERN).unwrap();

                let path: Vec<_> = regex
                    .captures_iter(trans)
                    .map(|m| m.get(2).unwrap().as_str())
                    .collect();

                let target = path.get(0).unwrap_or(&trans);

                let path = PathBuf::from(target);
                if path.is_relative() && File::open(&path).is_ok() {
                    Ok(Some(Node::Transclusion(Transclusion::new(path))))
                } else {
                    Err(ParseError::InvalidTransclusionError(format!(
                        "Not a relative path, of file not found: {}",
                        line.to_owned()
                    )))
                }
            } else {
                Err(ParseError::UnclosedTransclusionError(line.to_owned()))
            }
        } else {
            Ok(None)
        }
    }
}

impl ParserConfig for MdParser {
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
    fn blank_lines(&self) -> bool {
        self.blank_lines
    }
}

impl Parser for MdParser {
    type Error = MdError;

    fn parse(&self, input: &str) -> Result<Document, Self::Error> {
        #[derive(Default)]
        struct State {
            node: Option<Node>,
        }

        enum Parse {
            Incomplete,
            Complete(Node),
            Error(MdError),
        }

        let mut state = State::default();
        let mut document = input
            .lines()
            .enumerate()
            .scan(&mut state, |state, (line_number, line)| {
                let (is_code, is_alt_fenced_code) =
                    if let Some(Node::Code(code_block)) = &state.node {
                        (true, code_block.alternative)
                    } else {
                        (false, false)
                    };
                let starts_fenced_alt = line.trim_start().starts_with(&self.fence_sequence_alt);
                let starts_fenced = if starts_fenced_alt {
                    false
                } else {
                    line.trim_start().starts_with(&self.fence_sequence)
                };

                if ((!is_code) && (starts_fenced || starts_fenced_alt))
                    || (is_code && starts_fenced && !is_alt_fenced_code)
                    || (is_code && starts_fenced_alt && is_alt_fenced_code)
                {
                    let fence_sequence = if starts_fenced_alt {
                        &self.fence_sequence_alt
                    } else {
                        &self.fence_sequence
                    };
                    match state.node.take() {
                        Some(Node::Code(code_block)) => {
                            if line.starts_with(&code_block.indent) {
                                state.node = None;
                                Some(Parse::Complete(Node::Code(code_block)))
                            } else {
                                Some(Parse::Error(MdError::Single {
                                    line_number,
                                    kind: MdErrorKind::IncorrectIndentation,
                                }))
                            }
                        }
                        previous => {
                            let indent_len = line.find(fence_sequence).unwrap();
                            let (indent, rest) = line.split_at(indent_len);
                            let rest = &rest[fence_sequence.len()..];
                            let name_start = rest.find(&self.block_name_start);
                            let name = name_start
                                .map(|start| start + self.block_name_start.len())
                                .map(|name_start| {
                                    let name_end = self
                                        .block_name_end
                                        .as_ref()
                                        .and_then(|end| {
                                            rest[name_start + self.block_name_start.len()..]
                                                .find(end)
                                        })
                                        .map(|name_end| {
                                            self.block_name_start.len() + name_end + name_start
                                        })
                                        .unwrap_or(rest.len());
                                    let name = &rest[name_start..name_end];
                                    self.parse_name(name, false)
                                });

                            let mut code_block = CodeBlock::new().indented(indent);

                            let language = rest[..name_start.unwrap_or(rest.len())].trim();
                            let language = if language.is_empty() {
                                match &self.default_language {
                                    Some(language) => Some(language.to_owned()),
                                    None => None,
                                }
                            } else {
                                Some(language.to_owned())
                            };
                            if let Some(language) = language {
                                code_block = code_block.in_language(language);
                            }
                            code_block = code_block.alternative(starts_fenced_alt);
                            code_block = match name {
                                None => code_block,
                                Some(Ok((name, vars, defaults))) => {
                                    let hidden = name.starts_with(&self.hidden_prefix);
                                    let name = if hidden {
                                        &name[self.hidden_prefix.len()..]
                                    } else {
                                        &name[..]
                                    };
                                    code_block
                                        .named(name.to_string(), vars, defaults)
                                        .hidden(hidden)
                                }
                                Some(Err(error)) => {
                                    return Some(Parse::Error(MdError::Single {
                                        line_number,
                                        kind: error.into(),
                                    }))
                                }
                            };
                            state.node = Some(Node::Code(code_block));
                            match previous {
                                None => Some(Parse::Incomplete),
                                Some(node) => Some(Parse::Complete(node)),
                            }
                        }
                    }
                } else {
                    match &mut state.node {
                        None => {
                            let mut new_block = TextBlock::new();
                            new_block.add_line(line);
                            state.node = Some(Node::Text(new_block));
                            match self.parse_transclusion(line) {
                                Err(err) => Some(Parse::Error(MdError::Single {
                                    line_number,
                                    kind: MdErrorKind::Parse(err),
                                })),
                                Ok(trans) => match trans {
                                    Some(node) => {
                                        let new_block = TextBlock::new();
                                        state.node = Some(Node::Text(new_block));

                                        Some(Parse::Complete(node))
                                    }
                                    None => Some(Parse::Incomplete),
                                },
                            }
                            //Some(Parse::Incomplete)
                        }
                        Some(Node::Text(block)) => match self.parse_transclusion(line) {
                            Err(err) => Some(Parse::Error(MdError::Single {
                                line_number,
                                kind: MdErrorKind::Parse(err),
                            })),
                            Ok(trans) => match trans {
                                Some(node) => {
                                    let ret = state.node.take();
                                    state.node = Some(node);
                                    Some(Parse::Complete(ret.unwrap()))
                                }
                                None => {
                                    block.add_line(line);
                                    Some(Parse::Incomplete)
                                }
                            },
                        },
                        Some(Node::Code(block)) => {
                            if line.starts_with(&block.indent) {
                                if block.name.is_none()
                                    && block.source.is_empty()
                                    && line.trim().starts_with(&self.comment_start)
                                {
                                    let trim = line.trim()[self.comment_start.len()..].trim();
                                    let name = self.parse_name(trim, false);
                                    match name {
                                        Ok((name, vars, defaults)) => {
                                            let hidden = name.starts_with(&self.hidden_prefix);
                                            let name = if hidden {
                                                &name[self.hidden_prefix.len()..]
                                            } else {
                                                &name[..]
                                            };
                                            block.name = Some(name.to_string());
                                            block.hidden = hidden;
                                            block.vars = vars;
                                            block.defaults = defaults;
                                        }
                                        Err(error) => {
                                            return Some(Parse::Error(MdError::Single {
                                                line_number,
                                                kind: error.into(),
                                            }))
                                        }
                                    };
                                    Some(Parse::Incomplete)
                                } else {
                                    let line = match self
                                        .parse_line(line_number, &line[block.indent.len()..])
                                    {
                                        Ok(line) => line,
                                        Err(error) => {
                                            return Some(Parse::Error(MdError::Single {
                                                line_number,
                                                kind: error.into(),
                                            }))
                                        }
                                    };
                                    block.add_line(line);
                                    Some(Parse::Incomplete)
                                }
                            } else {
                                Some(Parse::Error(MdError::Single {
                                    line_number,
                                    kind: MdErrorKind::IncorrectIndentation,
                                }))
                            }
                        }
                        Some(Node::Transclusion(trans)) => {
                            let trans = trans.clone();
                            let mut new_block = TextBlock::new();
                            new_block.add_line(line);
                            state.node = Some(Node::Text(new_block));
                            Some(Parse::Complete(Node::Transclusion(trans)))
                        }
                    }
                }
            })
            .filter_map(|parse| match parse {
                Parse::Incomplete => None,
                Parse::Error(error) => Some(Err(error)),
                Parse::Complete(node) => Some(Ok(node)),
            })
            .try_collect::<_, _, Vec<_>, MdError>()?;
        if let Some(node) = state.node.take() {
            document.push(node);
        }
        Ok(Document::from_iter(document))
    }

    fn find_links(&self, input: &Document) -> Result<Vec<PathBuf>, Self::Error> {
        let regex = Regex::new(Self::LINK_PATTERN).unwrap();
        let paths = input
            .tree()
            .text_blocks()
            .iter()
            .flat_map(|block| {
                block.lines().iter().flat_map(|line| {
                    regex
                        .captures_iter(line)
                        .map(|m| m.get(2).unwrap().as_str())
                        .filter_map(|p| {
                            let path = PathBuf::from(p);
                            if path.is_relative() && !p.starts_with("#") {
                                if File::open(&path).is_ok() {
                                    Some(path)
                                } else {
                                    // TODO: move out of function?
                                    eprintln!("WARNING: link target not found for {:?}", path);
                                    None
                                }
                            } else {
                                None
                            }
                        })
                })
            })
            .collect();
        Ok(paths)
    }
}

impl Printer for MdParser {
    fn print_text_block(&self, block: &TextBlock) -> String {
        format!("{}\n", block.to_string())
    }

    fn print_code_block(&self, block: &CodeBlock) -> String {
        let fence_sequence = if block.alternative {
            &self.fence_sequence_alt
        } else {
            &self.fence_sequence
        };
        let mut output = fence_sequence.clone();
        if let Some(language) = &block.language {
            output.push_str(language);
        }
        output.push('\n');
        if let Some(name) = &block.name {
            output.push_str(&self.comment_start);
            output.push(' ');
            output.push_str(&self.print_name(name.clone(), &block.vars, &block.defaults));
            output.push('\n');
        }

        let mut comments = vec![];
        let line_offset = block.line_number().unwrap_or(0);
        for line in &block.source {
            output.push_str(&self.print_line(&line, !self.comments_as_aside));
            if self.comments_as_aside {
                if let Some(comment) = &line.comment {
                    comments.push((line.line_number - line_offset, comment));
                }
            }
            output.push('\n');
        }

        output.push_str(fence_sequence);
        output.push('\n');

        for (line, comment) in comments {
            output.push_str(&format!(
                "<aside class=\"comment\" data-line=\"{}\">{}</aside>\n",
                line,
                comment.trim()
            ));
        }

        output
    }

    fn print_transclusion(&self, transclusion: &Transclusion) -> String {
        let mut output = String::new();
        output.push_str("**WARNING!** Missed/skipped transclusion: ");
        output.push_str(transclusion.file().to_str().unwrap());
        output.push('\n');
        output
    }
}

/// Kinds of errors that can be encountered while parsing and restructuring the Markdown
#[derive(Debug)]
pub enum MdErrorKind {
    /// A line was un-indented too far, usually indicating an error
    IncorrectIndentation,
    /// Generic parse error
    Parse(ParseError),
}

/// Errors that were encountered while parsing the HTML
#[derive(Debug)]
pub enum MdError {
    #[doc(hidden)]
    Single {
        line_number: usize,
        kind: MdErrorKind,
    },
    #[doc(hidden)]
    Multi(Vec<MdError>),
}

impl std::fmt::Display for MdError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MdError::Multi(errors) => {
                for error in errors {
                    writeln!(f, "{}", error)?;
                }
                Ok(())
            }
            MdError::Single { line_number, kind } => {
                writeln!(f, "{:?} (line {})", kind, line_number)
            }
        }
    }
}

impl std::error::Error for MdError {}

impl FromIterator<MdError> for MdError {
    fn from_iter<I: IntoIterator<Item = MdError>>(iter: I) -> Self {
        MdError::Multi(iter.into_iter().collect())
    }
}

impl From<Vec<MdError>> for MdError {
    fn from(multi: Vec<MdError>) -> Self {
        MdError::Multi(multi)
    }
}

impl From<ParseError> for MdErrorKind {
    fn from(error: ParseError) -> Self {
        MdErrorKind::Parse(error)
    }
}
