//! Representation of the code parts of the AST

use super::{CompileError, CompileErrorKind};
use crate::config::LanguageSettings;
use crate::util::TryCollectExt;
use std::collections::HashMap;
use std::fmt::Write;
use std::path::PathBuf;

/// A `CodeBlock` is a block of code as defined by the input format.
#[derive(Clone, Default, Debug)]
pub struct CodeBlock {
    /// The indent of this code block is in the documentation file
    pub indent: String,
    /// The name of this code block
    pub name: Option<String>,
    /// Whether the code block was originally unnamed
    pub is_unnamed: bool,
    /// The language this block was written in
    pub language: Option<String>,
    /// Marks the code block as hidden from docs
    pub hidden: bool,
    /// Marks the code block as fenced by alternative sequence
    pub alternative: bool,
    /// The source is the lines of code
    pub source: Vec<Line>,
    /// Source file, for transcluded blocks
    pub source_file: Option<PathBuf>,
}

impl CodeBlock {
    /// Creates a new empty `CodeBlock`
    pub fn new() -> Self {
        Self::default()
    }

    /// Indents this code block
    pub fn indented(self, indent: &str) -> Self {
        Self {
            indent: indent.to_owned(),
            ..self
        }
    }

    /// Names this code block
    pub fn named(self, name: String) -> Self {
        Self {
            name: Some(name),
            ..self
        }
    }

    /// Hides this code block
    pub fn hidden(self, hidden: bool) -> Self {
        Self { hidden, ..self }
    }

    /// Marks the code block as fenced by alternative sequence
    pub fn alternative(self, alternative: bool) -> Self {
        Self {
            alternative,
            ..self
        }
    }

    /// Sets the language of this code block
    pub fn in_language(self, language: String) -> Self {
        Self {
            language: Some(language),
            ..self
        }
    }

    /// Adds a line to this code block
    pub fn add_line(&mut self, line: Line) {
        self.source.push(line);
    }

    /// Appends another code block to the end of this one
    pub fn append(&mut self, other: &CodeBlock) {
        self.source.extend_from_slice(&other.source)
    }

    /// "Compiles" this code block into its output code
    pub fn compile(
        &self,
        code_blocks: &HashMap<Option<&str>, Vec<&CodeBlock>>,
        settings: Option<&LanguageSettings>,
    ) -> Result<String, CompileError> {
        self.compile_with(code_blocks, settings)
    }

    /// Returns the line number of the first line in this code block
    pub fn line_number(&self) -> Option<usize> {
        self.source.first().map(|line| line.line_number)
    }

    fn compile_with(
        &self,
        code_blocks: &HashMap<Option<&str>, Vec<&CodeBlock>>,
        settings: Option<&LanguageSettings>,
    ) -> Result<String, CompileError> {
        self.source
            .iter()
            .map(|line| line.compile_with(code_blocks, settings))
            .try_collect()
            .map(|lines| lines.join("\n"))
            .map_err(CompileError::Multi)
    }
}

/// A `Source` represents the source code on a line.
#[derive(Clone, Debug)]
pub enum Source {
    /// A macro invocation, resolved by the literate compiler
    Macro(String),
    /// Source text, possibly including meta variable interpolations
    Source(String),
}

/// A `Line` defines a line of code.
#[derive(Clone, Debug)]
pub struct Line {
    /// The line number of this line (original source)
    pub line_number: usize,
    /// The indent on this line. An indent is defined as leading whitespace (spaces/tabs)
    pub indent: String,
    /// The source text
    pub source: Source,
    /// The literate compiler defined comment - this is extracted from source text to be rendered
    /// in an appropriate format in the documentation, rather than as a comment in the source file
    pub comment: Option<String>,
}

impl Line {
    fn compile_with(
        &self,
        code_blocks: &HashMap<Option<&str>, Vec<&CodeBlock>>,
        settings: Option<&LanguageSettings>,
    ) -> Result<String, CompileError> {
        let comment_start = settings
            .and_then(|s| s.block_labels.as_ref())
            .map(|l| l.comment_start.as_str())
            .unwrap_or_default();
        let comment_end = settings
            .and_then(|s| s.block_labels.as_ref())
            .and_then(|l| l.comment_end.as_deref())
            .unwrap_or_default();
        let block_start = settings
            .and_then(|s| s.block_labels.as_ref())
            .map(|l| l.block_start.as_str())
            .unwrap_or_default();
        let block_end = settings
            .and_then(|s| s.block_labels.as_ref())
            .map(|l| l.block_end.as_str())
            .unwrap_or_default();
        let block_next = settings
            .and_then(|s| s.block_labels.as_ref())
            .map(|l| l.block_next.as_str())
            .unwrap_or("");
        let block_name_sep = '#';

        let clean = if let Some(s) = settings {
            s.clean_code || s.block_labels.is_none()
        } else {
            true
        };

        let blank_lines = settings.map(|s| s.clear_blank_lines).unwrap_or(true);
        match &self.source {
            Source::Source(string) => {
                if blank_lines && string.trim().is_empty() {
                    Ok("".to_string())
                } else {
                    Ok(format!("{}{}", self.indent, string))
                }
            }
            Source::Macro(name) => {
                let blocks = code_blocks.get(&Some(name)).ok_or(CompileError::Single {
                    line_number: self.line_number,
                    kind: CompileErrorKind::UnknownMacro(name.to_string()),
                })?;

                let mut result = String::new();
                for (idx, block) in blocks.iter().enumerate() {
                    let path = block.source_file.to_owned().unwrap_or_default();
                    let name = if block.is_unnamed {
                        ""
                    } else {
                        block.name.as_ref().map(|n| &n[..]).unwrap_or("")
                    };

                    if !clean {
                        writeln!(
                            result,
                            "{}{} {}{}{}{}{}{}{}",
                            &self.indent,
                            comment_start,
                            if idx == 0 { &block_start } else { &block_next },
                            path.display(),
                            block_name_sep,
                            name,
                            block_name_sep,
                            idx,
                            comment_end,
                        )
                        .unwrap();
                    }

                    let code = block.compile_with(code_blocks, settings)?;
                    for line in code.lines() {
                        if blank_lines && line.trim().is_empty() {
                            writeln!(result).unwrap();
                        } else {
                            write!(result, "{}", self.indent).unwrap();
                            writeln!(result, "{}", line).unwrap();
                        }
                    }

                    if !clean && idx == blocks.len() - 1 {
                        writeln!(
                            result,
                            "{}{} {}{}{}{}{}{}{}",
                            &self.indent,
                            comment_start,
                            &block_end,
                            path.display(),
                            block_name_sep,
                            name,
                            block_name_sep,
                            idx,
                            comment_end,
                        )
                        .unwrap();
                    }
                }
                result.pop();
                Ok(result)
            }
        }
    }
}
