//! The parser for Markdown based literate programming.
//!
//! This includes some extensions to support some of the more advanced features of this tool.
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
//! All code blocks with the same name are concatenated, in the order
//! they are found, and the "unnamed" block is used as the entry point when generating the output
//! source file. Any code blocks with names which are not invoked will not appear in the compiled
//! code.
//!
//! Currently, the Markdown parser does not support code that is written to the compiled file, but
//! not rendered in the documentation file.

use super::code::RevCodeBlock;

use crate::document::{
    code::{CodeBlock, Line, Source},
    text::TextBlock,
    transclusion::Transclusion,
    Document, Node,
};
use crate::util::{Fallible, TryCollectExt};
use once_cell::sync::Lazy;
use regex::Regex;
use serde::de::Error as _;
use serde::{Deserialize, Deserializer};
use std::collections::HashMap;
use std::error::Error;
use std::fmt::Write;
use std::fs::File;
use std::path::{Path, PathBuf};

/// The config for parsing a Markdown document
#[derive(Clone, Deserialize, Debug)]
pub struct MdParser {
    /// The sequence that identifies the start and end of a fenced code block
    pub fence_sequence: String,
    /// Alternative sequence that identifies the start and end of a fenced code block.
    /// Allows for normal Markdown fences in code blocks
    pub fence_sequence_alt: String,
    /// Parsed comments are stripped from the code and written to an `<aside></aside>` block after
    /// the code when printing. If false, the comments are just written back into the code.
    pub comments_as_aside: bool,
    /// The language to set if there was no automatically detected language. Optional
    pub default_language: Option<String>,
    /// The sequence to identify a comment which should be omitted from the compiled code, and may
    /// be rendered as an `<aside>` if `comments_as_aside` is set.
    pub comment_start: String,
    /// The sequence to identify the start of a macro invocation.
    pub macro_start: String,
    /// The sequence to identify the end of a macro invocation.
    pub macro_end: String,
    /// The sequence to identify the start of a transclusion.
    pub transclusion_start: String,
    /// The sequence to identify the end of a transclusion.
    pub transclusion_end: String,
    /// Prefix for links that should be followed during processing.
    /// Should be RegEx-compatible.
    #[serde(rename(deserialize = "link_prefix"))]
    #[serde(deserialize_with = "from_link_prefix")]
    pub link_following_pattern: (String, Regex),
    /// Prefix for file-specific entry points.
    pub file_prefix: String,
    /// Name prefix for code blocks not shown in the docs.
    pub hidden_prefix: String,
}

const LINK_PATTERN: &str = r"\[([^\[\]]*)\]\((.*?)\)";

static LINK_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(LINK_PATTERN).unwrap());

fn from_link_prefix<'de, D>(deserializer: D) -> Result<(String, Regex), D::Error>
where
    D: Deserializer<'de>,
{
    let prefix: &str = Deserialize::deserialize(deserializer)?;
    Ok((
        prefix.to_string(),
        Regex::new(&format!("{}{}", prefix, LINK_PATTERN)).map_err(|err| {
            D::Error::custom(format!(
                "Error compiling Regex pattern {}{}\n{}",
                prefix,
                LINK_PATTERN,
                err.to_string()
            ))
        })?,
    ))
}

impl MdParser {
    /// Sets the default language of the returned parser (or does nothing if `None` is passed)
    pub fn default_language(&self, language: Option<String>) -> Self {
        let mut cloned = self.clone();

        if language.is_some() {
            cloned.default_language = language;
        }

        cloned
    }

    fn parse_transclusion(&self, line: &str, into: &Path) -> Fallible<Option<Node>> {
        if let Some(rest) = line.trim().strip_prefix(&self.transclusion_start) {
            if let Some(index) = rest.find(&self.transclusion_end) {
                let trans = &rest[..index];

                let target = LINK_REGEX
                    .captures_iter(trans)
                    .map(|match_| match_.get(2).unwrap().as_str())
                    .next()
                    .unwrap_or(&trans);

                let mut path = PathBuf::from(into.parent().unwrap_or_else(|| Path::new(".")));
                path.push(target);

                Ok(Some(Node::Transclusion(Transclusion::new(
                    path,
                    line.to_owned(),
                ))))
            } else {
                Err(format!("Unclosed transclusion in: {}", line).into())
            }
        } else {
            Ok(None)
        }
    }

    #[allow(clippy::nonminimal_bool)]
    pub fn parse(&self, input: &str, path: &Path) -> Fallible<Document> {
        #[derive(Default)]
        struct State {
            node: Option<Node>,
        }

        enum Parse {
            Incomplete,
            Complete(Node),
            Error(Box<dyn Error>),
        }

        impl Parse {
            fn error(err: Box<dyn Error>, line: usize) -> Self {
                Self::Error(format!("{} (line {})", err, line).into())
            }
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

                if (!is_code && (starts_fenced || starts_fenced_alt))
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
                                Some(Parse::Error(
                                    format!("Incorrect indentation in line {}", line_number).into(),
                                ))
                            }
                        }
                        previous => {
                            let indent_len = line.find(fence_sequence).unwrap();
                            let (indent, rest) = line.split_at(indent_len);
                            let rest = &rest[fence_sequence.len()..];

                            let mut code_block = CodeBlock::new().indented(indent);

                            let language = rest.trim();
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
                            match self.parse_transclusion(line, path) {
                                Err(err) => Some(Parse::error(err, line_number)),
                                Ok(trans) => match trans {
                                    Some(node) => {
                                        let new_block = TextBlock::new();
                                        state.node = Some(Node::Text(new_block));

                                        Some(Parse::Complete(node))
                                    }
                                    None => Some(Parse::Incomplete),
                                },
                            }
                        }
                        Some(Node::Text(block)) => match self.parse_transclusion(line, path) {
                            Err(err) => Some(Parse::error(err, line_number)),
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
                                if block.source.is_empty()
                                    && line.trim().starts_with(&self.comment_start)
                                {
                                    let mut name = line.trim()[self.comment_start.len()..].trim();
                                    let hidden = if let Some(stripped) =
                                        name.strip_prefix(&self.hidden_prefix)
                                    {
                                        name = stripped;
                                        true
                                    } else {
                                        false
                                    };

                                    block.name = Some(name.to_string());
                                    block.hidden = hidden;

                                    Some(Parse::Incomplete)
                                } else {
                                    let line = match self
                                        .parse_line(line_number, &line[block.indent.len()..])
                                    {
                                        Ok(line) => line,
                                        Err(error) => {
                                            return Some(Parse::error(error, line_number));
                                        }
                                    };
                                    block.add_line(line);
                                    Some(Parse::Incomplete)
                                }
                            } else {
                                Some(Parse::Error(
                                    format!("Incorrect indentation line {}", line_number).into(),
                                ))
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
            .try_collect()
            .map_err(|errors| {
                let mut msg = String::new();
                for error in errors {
                    writeln!(&mut msg, "{}", error).unwrap();
                }
                msg
            })?;
        if let Some(node) = state.node.take() {
            document.push(node);
        }
        Ok(Document::new(document))
    }

    pub fn find_links(
        &self,
        input: &mut Document,
        from: &Path,
        remove_marker: bool,
    ) -> Fallible<Vec<PathBuf>> {
        let regex = &self.link_following_pattern;
        let mut paths = vec![];

        for block in input.text_blocks_mut() {
            for line in block.lines_mut().iter_mut() {
                let mut offset = 0;
                let mut new_line: Option<String> = None;
                for capture in regex.1.captures_iter(line) {
                    if remove_marker {
                        let index = capture.get(0).unwrap().start();
                        let len = regex.0.len();
                        if let Some(l) = &mut new_line {
                            *l = format!(
                                "{}{}",
                                &l[..(index - offset)],
                                &l[(index + len - offset)..]
                            );
                        } else {
                            new_line = Some(format!(
                                "{}{}",
                                &line[..(index - offset)],
                                &line[(index + len - offset)..]
                            ));
                        }
                        offset += len;
                    }

                    let link = capture.get(2).unwrap().as_str();
                    let mut path = from.parent().unwrap().to_path_buf();
                    path.push(link);
                    let path = PathBuf::from(path_clean::clean(
                        &path.to_str().unwrap().replace("\\", "/"),
                    ));
                    if path.is_relative()
                        && !link.starts_with('#')
                        && !link.starts_with("http://")
                        && !link.starts_with("https://")
                        && !link.starts_with("ftp://")
                    {
                        if File::open(&path).is_ok() {
                            paths.push(path);
                        } else {
                            // TODO: move out of function?
                            eprintln!("WARNING: link target not found for {}", path.display());
                        }
                    }
                }
                if let Some(new_line) = new_line {
                    *line = new_line;
                }
            }
        }
        Ok(paths)
    }

    /// Parses a line as code, returning the parsed `Line` object
    fn parse_line(&self, line_number: usize, input: &str) -> Fallible<Line> {
        let indent_len = input
            .chars()
            .take_while(|ch| ch.is_whitespace())
            .collect::<String>()
            .len();
        let (indent, rest) = input.split_at(indent_len);
        let (rest, comment) = if let Some(comment_index) = rest.find(&self.comment_start) {
            let (rest, comment) = rest.split_at(comment_index);
            (
                rest,
                Some((&comment[self.comment_start.len()..]).to_owned()),
            )
        } else {
            (rest, None)
        };

        if let Some(stripped) = rest.strip_prefix(&self.macro_start) {
            if let Some(name) = stripped.strip_suffix(&self.macro_end) {
                return Ok(Line {
                    line_number,
                    indent: indent.to_owned(),
                    source: Source::Macro(name.trim().to_owned()),
                    comment,
                });
            }
        }

        Ok(Line {
            line_number,
            indent: indent.to_owned(),
            source: Source::Source(rest.to_owned()),
            comment,
        })
    }

    /// Finds all file-specific entry points
    pub fn get_entry_points<'a>(
        &self,
        doc: &'a Document,
        language: Option<&'a str>,
    ) -> HashMap<Option<&'a str>, &'a Path> {
        let mut entries = HashMap::new();
        let pref = &self.file_prefix;
        for block in doc.code_blocks(language) {
            if let Some(name) = block.name.as_deref() {
                if let Some(rest) = name.strip_prefix(pref) {
                    entries.insert(Some(name), Path::new(rest));
                }
            }
        }
        entries
    }
}

impl MdParser {
    pub fn print_code_block(&self, block: &CodeBlock) -> String {
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
            output.push_str(name);
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

    pub fn print_code_block_reverse(
        &self,
        block: &CodeBlock,
        alternative: Option<&RevCodeBlock>,
    ) -> String {
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
            if block.hidden {
                output.push_str(&self.hidden_prefix);
            }
            output.push_str(name);
            output.push('\n');
        }

        if let Some(alt) = alternative {
            for line in &alt.lines {
                output.push_str(&line);
                output.push('\n');
            }
        } else {
            for line in &block.source {
                output.push_str(&self.print_line(&line, true));
                output.push('\n');
            }
        }

        output.push_str(fence_sequence);
        output.push('\n');

        output
    }

    pub fn print_text_block(&self, block: &TextBlock) -> String {
        format!("{}\n", block.to_string())
    }

    pub fn print_transclusion(&self, transclusion: &Transclusion, reverse: bool) -> String {
        let mut output = String::new();
        if reverse {
            output.push_str(transclusion.original());
            output.push('\n');
        } else {
            output.push_str("**WARNING!** Missed/skipped transclusion: ");
            output.push_str(transclusion.file().to_str().unwrap());
            output.push('\n');
        }
        output
    }

    /// Prints a line of a code block
    pub fn print_line(&self, line: &Line, print_comments: bool) -> String {
        let mut output = line.indent.to_string();
        match &line.source {
            Source::Macro(name) => {
                output.push_str(&self.macro_start);
                if !self.macro_start.ends_with(' ') {
                    output.push(' ');
                }
                output.push_str(name);
                output.push_str(&self.macro_end);
            }
            Source::Source(string) => {
                output.push_str(string);
            }
        }
        if print_comments {
            if let Some(comment) = &line.comment {
                output.push_str(&self.comment_start);
                output.push_str(&comment);
            }
        }
        output
    }
}
