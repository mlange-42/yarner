//! The parser for HTML based literate programming.
//!
//! See `examples/html/wc.c.html` for an example of how to use this format with the default config,
//! which is specified as follows:
//!
//! *   Code is identified by a `<code>` tag.
//! *   The code tag supports `language` and `name` attributes to specify the language and name.
//!     Any other attributes will be ignored, or may even fail to parse.
//! *   When printing the documentation, the language and name are written in the `data-language`
//!     and `data-name` attributes, to be consistent with valid HTML.
//! *   The language will also be added as a class, such as `language-c`.
//! *   When printing the documentation, the `block` class will be added to any detected code
//!     blocks.
//! *   The comment symbol is `//`, but they are rendered inline
//! *   Interpolation of is done such as `@{a meta variable}`.
//! *   Macros (named code blocks) are invoked by `==> Macro name.` (note the period at the end)
//!
//! As with all supported styles, all code blocks with the same name are concatenated, in the order
//! they are found, and the "unnamed" block is used as the entry point when generating the output
//! source file. Any code blocks with names which are not invoked will not appear in the compiled
//! code.
//!
//! HTML entities inside the code blocks should not be HTML escaped - they will be escaped
//! automatically in the output documentation file.
//!
//! Note that due to the "stupid" parsing, the opening and closing `<code>` tags must be specified
//! on their own line for them to be parsed for compilation as code. Inline code tags (with other
//! text on the line) are just interpreted as text.
//!
//! Currently, the HTML parser does not support code that is written to the compiled file, but
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

/// The config for parsing an HTML document
#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct HtmlParser {
    /// The tag that code is written in when parsing. When printing, the code is always rendered in
    /// `<pre><code></code></pre>`, as is recommended by W3C.
    ///
    /// Default: `code`
    pub code_tag: String,
    /// The attribute to put the language name when rendering the documentation.
    ///
    /// Default: `data-language`
    ///
    /// When parsing, the language is always read from the `language` attribute (this value is
    /// ignored).
    pub language_attribute: String,
    /// The attribute to add to the `code_tag` with the code block's name .
    ///
    /// Default: `data-name`
    ///
    /// When parsing, the name is always read from the `name` attribute (this value is ignored)
    pub name_attribute: String,
    /// A class to add to the `<pre>` container for any parsed blocks of code when rendering.
    pub block_class: String,
    /// The class to add to the `<code>` block when rendering. The string `{}` is replaced with the
    /// language that this block is in.
    ///
    /// Default: `language-{}`
    pub language_class: String,
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

impl Default for HtmlParser {
    fn default() -> Self {
        Self {
            default_language: None,
            code_tag: String::from("code"),
            language_class: String::from("language-{}"),
            language_attribute: String::from("data-language"),
            name_attribute: String::from("data-name"),
            block_class: String::from("block"),
            comments_as_aside: false,
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

impl HtmlParser {
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

impl ParserConfig for HtmlParser {
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

impl HtmlParser {
    fn parse_arguments<'a>(
        &self,
        arg_string: &'a str,
    ) -> Result<HashMap<&'a str, &'a str>, HtmlErrorKind> {
        let mut args = HashMap::new();
        let mut arg_string = arg_string.trim();
        while arg_string != ">" {
            if arg_string.is_empty() {
                return Err(HtmlErrorKind::InvalidArgumentList);
            }
            let name_len = arg_string
                .chars()
                .take_while(|ch| ch.is_alphabetic())
                .collect::<String>()
                .len();
            if name_len == 0 {
                return Err(HtmlErrorKind::ExtraCharactersInCodeBlock);
            }
            let (name, rest) = arg_string.split_at(name_len);
            let rest = if rest.trim_start().starts_with('=') {
                let equals = rest.find('=').unwrap();
                let rest = rest[equals + 1..].trim_start();
                let (value, rest) = if rest.starts_with('"') {
                    let rest = &rest[1..];
                    let value_len = rest
                        .chars()
                        .scan('"', |state, ch| {
                            let previous = std::mem::replace(state, ch);
                            if previous == '\\' {
                                Some(ch)
                            } else if ch == '"' {
                                None
                            } else {
                                Some(ch)
                            }
                        })
                        .collect::<String>()
                        .len();
                    if value_len == rest.len() {
                        return Err(HtmlErrorKind::UnmatchedQuoteInArgumentList);
                    }
                    let (value, rest) = rest.split_at(value_len);
                    (value, &rest[1..])
                } else {
                    let ws = match rest.find(char::is_whitespace) {
                        Some(index) => index,
                        None => rest.len(),
                    };
                    rest.split_at(ws)
                };
                args.insert(name, value);
                rest
            } else {
                args.insert(name, name);
                rest
            };
            arg_string = rest.trim_start();
        }
        Ok(args)
    }
}

impl Parser for HtmlParser {
    type Error = HtmlError;

    fn parse(&self, input: &str) -> Result<Document, Self::Error> {
        struct State {
            node: Node,
        }

        enum Parse {
            Incomplete,
            Complete(Node),
            Error(HtmlError),
        }

        let tag_open = format!("<{}", self.code_tag);
        let tag_close = format!("</{}>", self.code_tag);

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
                            return Some(Parse::Error(HtmlError::Single {
                                line_number,
                                kind: HtmlErrorKind::IncorrectIndentation,
                            }));
                        }
                        let line = &line[code_block.indent.len()..];
                        if line.starts_with(&tag_close) {
                            let node =
                                std::mem::replace(&mut state.node, Node::Text(TextBlock::new()));
                            Some(Parse::Complete(node))
                        } else {
                            let line = match self.parse_line(line_number, line) {
                                Ok(line) => line,
                                Err(error) => {
                                    return Some(Parse::Error(HtmlError::Single {
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
                        if line.trim_start().starts_with(&tag_open) {
                            let start = line.find(&tag_open).unwrap();
                            let (indent, rest) = line.split_at(start);
                            let rest = &rest[tag_open.len()..].trim();
                            let args = match self.parse_arguments(rest) {
                                Ok(args) => args,
                                Err(HtmlErrorKind::ExtraCharactersInCodeBlock) => {
                                    text_block.add_line(line);
                                    return Some(Parse::Incomplete);
                                }
                                Err(kind) => {
                                    return Some(Parse::Error(HtmlError::Single {
                                        line_number,
                                        kind,
                                    }))
                                }
                            };
                            let mut code_block = CodeBlock::new().indented(indent);

                            if let Some(name) = args.get("name") {
                                let (name, vars, defaults) = match self.parse_name(name, false) {
                                    Ok(name) => name,
                                    Err(error) => {
                                        return Some(Parse::Error(HtmlError::Single {
                                            line_number,
                                            kind: error.into(),
                                        }))
                                    }
                                };
                                let hidden = name.starts_with(&self.hidden_prefix);
                                code_block = code_block.named(name, vars, defaults).hidden(hidden);
                            }
                            code_block = match args.get("language") {
                                Some(language) => code_block.in_language((*language).to_string()),
                                None => match &self.default_language {
                                    Some(language) => {
                                        code_block.in_language((*language).to_string())
                                    }
                                    None => code_block,
                                },
                            };
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
            .try_collect::<_, _, Vec<_>, HtmlError>()?;
        document.push(state.node);
        Ok(Document::from_iter(document))
    }

    fn find_links(&self, _input: &Document, _from: &PathBuf) -> Result<Vec<PathBuf>, Self::Error> {
        Ok(vec![])
    }
}

impl Printer for HtmlParser {
    fn print_text_block(&self, block: &TextBlock) -> String {
        format!("{}\n", block.to_string())
    }

    fn print_code_block(&self, block: &CodeBlock) -> String {
        let mut output = String::new();
        output.push_str(&format!("<pre class=\"{}\"><code", self.block_class));
        if let Some(language) = &block.language {
            let class = self.language_class.replace("{}", language);
            output.push_str(&format!(
                " class=\"{}\" {}=\"{}\"",
                class, self.language_attribute, language
            ));
        }
        if let Some(name) = &block.name {
            output.push_str(&format!(
                " {}=\"{}\"",
                self.name_attribute,
                self.print_name(name.clone(), &block.vars, &block.defaults)
            ));
        }
        output.push_str(">\n");

        let mut comments = vec![];
        let line_offset = block.line_number().unwrap_or(0);
        for line in &block.source {
            output.push_str(
                &self
                    .print_line(&line, !self.comments_as_aside)
                    .replace("&", "&amp;")
                    .replace("<", "&lt;")
                    .replace(">", "&gt;"),
            );
            if self.comments_as_aside {
                if let Some(comment) = &line.comment {
                    comments.push((line.line_number - line_offset, comment));
                }
            }
            output.push('\n');
        }
        output.push_str("</code></pre>\n");

        for (line, comment) in comments {
            output.push_str(&format!(
                "<aside class=\"comment\" data-line=\"{}\">{}</aside>\n",
                line,
                comment.trim()
            ));
        }

        output
    }

    fn print_transclusion(&self, _transclusion: &Transclusion) -> String {
        // TODO
        String::new()
    }
}

/// Kinds of errors that can be encountered while parsing and restructuring the HTML
#[derive(Debug)]
pub enum HtmlErrorKind {
    /// A line was un-indented too far, usually indicating an error
    IncorrectIndentation,
    /// The quotation marks around an attribute value are mismatched
    UnmatchedQuoteInArgumentList,
    #[doc(hidden)]
    ExtraCharactersInCodeBlock,
    /// There is a syntax error in the code tag
    InvalidArgumentList,
    /// Generic parse error
    Parse(ParseError),
}

/// Errors that were encountered while parsing the HTML
#[derive(Debug)]
pub enum HtmlError {
    #[doc(hidden)]
    Single {
        line_number: usize,
        kind: HtmlErrorKind,
    },
    #[doc(hidden)]
    Multi(Vec<HtmlError>),
}

impl std::fmt::Display for HtmlError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HtmlError::Multi(errors) => {
                for error in errors {
                    writeln!(f, "{}", error)?;
                }
                Ok(())
            }
            HtmlError::Single { line_number, kind } => {
                writeln!(f, "{:?} (line {})", kind, line_number)
            }
        }
    }
}

impl std::error::Error for HtmlError {}

impl FromIterator<HtmlError> for HtmlError {
    fn from_iter<I: IntoIterator<Item = HtmlError>>(iter: I) -> Self {
        HtmlError::Multi(iter.into_iter().collect())
    }
}

impl From<Vec<HtmlError>> for HtmlError {
    fn from(multi: Vec<HtmlError>) -> Self {
        HtmlError::Multi(multi)
    }
}

impl From<ParseError> for HtmlErrorKind {
    fn from(error: ParseError) -> Self {
        HtmlErrorKind::Parse(error)
    }
}
