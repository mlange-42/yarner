//! Representation of the code parts of the AST

use super::{CompileError, CompileErrorKind};
use crate::config::LanguageSettings;
use crate::util::TryCollectExt;
use std::collections::HashMap;

/// A `CodeBlock` is a block of code as defined by the input format.
#[derive(Clone, Default, Debug)]
pub struct CodeBlock {
    /// The indent of this code block is in the documentation file
    pub indent: String,
    /// The name of this code block
    pub name: Option<String>,
    /// Whether the code block was originally unnamed
    pub is_unnamed: bool,
    /// The variables extracted from the name
    pub vars: Vec<String>,
    /// The variables' default values extracted from the name
    pub defaults: Vec<Option<String>>,
    /// The language this block was written in
    pub language: Option<String>,
    /// Marks the code block as hidden from docs
    pub hidden: bool,
    /// Marks the code block as fenced by alternative sequence
    pub alternative: bool,
    /// The source is the lines of code
    pub source: Vec<Line>,
    /// Source file, for transcluded blocks
    pub source_file: Option<String>,
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
    pub fn named(self, name: String, vars: Vec<String>, defaults: Vec<Option<String>>) -> Self {
        Self {
            name: Some(name),
            vars,
            defaults,
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
        self.compile_with(code_blocks, HashMap::default(), settings)
    }

    /// Returns the line number of the first line in this code block
    pub fn line_number(&self) -> Option<usize> {
        self.source.first().map(|line| line.line_number)
    }

    fn compile_with(
        &self,
        code_blocks: &HashMap<Option<&str>, Vec<&CodeBlock>>,
        scope: HashMap<String, String>,
        settings: Option<&LanguageSettings>,
    ) -> Result<String, CompileError> {
        self.source
            .iter()
            .map(|line| line.compile_with(code_blocks, &scope, settings))
            .try_collect()
            .map(|lines| lines.join("\n"))
            .map_err(CompileError::Multi)
    }

    fn assign_vars(&self, scope: &[String]) -> HashMap<String, String> {
        self.vars
            .iter()
            .zip(&self.defaults)
            .zip(scope)
            .map(|((name, default), value)| {
                (
                    name.clone(),
                    if value.is_empty() {
                        default.clone().unwrap_or_else(|| value.clone())
                    } else {
                        value.clone()
                    },
                )
            })
            .collect()
    }
}

/// A `Segment` is some of the raw source text.
#[derive(Clone, Debug)]
pub enum Segment {
    /// Raw source text
    Source(String),
    /// A meta variable, to be interpolated by the literate compiler
    MetaVar(String),
}

/// A `Source` represents the source code on a line.
#[derive(Clone, Debug)]
pub enum Source {
    /// A macro invocation, resolved by the literate compiler
    Macro {
        /// The name of the macro
        name: String,
        /// The meta-variable values to interpolate
        scope: Vec<String>,
    },
    /// Source text, possibly including meta variable interpolations
    Source(Vec<Segment>),
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
        scope: &HashMap<String, String>,
        settings: Option<&LanguageSettings>,
    ) -> Result<String, CompileError> {
        let comment_start = settings.map(|s| &s.comment_start[..]).unwrap_or("");
        let comment_end = settings
            .and_then(|s| s.comment_end.as_ref().map(|e| &e[..]))
            .unwrap_or("");
        let block_start = settings.map(|s| &s.block_start[..]).unwrap_or("");
        let block_end = settings.map(|s| &s.block_end[..]).unwrap_or("");
        let block_next = settings.map(|s| &s.block_next[..]).unwrap_or("");

        let clean = if let Some(s) = settings {
            s.clean_code
        } else {
            true
        };

        let blank_lines = settings.map(|s| s.clear_blank_lines).unwrap_or(true);
        match &self.source {
            Source::Source(segments) => {
                let code = segments
                    .iter()
                    .map(|segment| match segment {
                        Segment::Source(source) => Ok(source.clone()),
                        Segment::MetaVar(name) => scope
                            .get(&name[..])
                            .map(|var| var.to_owned())
                            .ok_or(CompileError::Single {
                                line_number: self.line_number,
                                kind: CompileErrorKind::UnknownMetaVariable(name.to_string()),
                            }),
                    })
                    .try_collect()
                    .map(|segments| segments.join(""))
                    .map_err(CompileError::Multi)?;

                if blank_lines && code.trim().is_empty() {
                    Ok("".to_string())
                } else {
                    Ok(format!("{}{}", self.indent, code))
                }
            }
            Source::Macro { name, scope } => {
                let blocks = code_blocks.get(&Some(name)).ok_or(CompileError::Single {
                    line_number: self.line_number,
                    kind: CompileErrorKind::UnknownMacro(name.to_string()),
                })?;

                let mut result = vec![];
                for (idx, block) in blocks.iter().enumerate() {
                    let scope = block.assign_vars(&scope[..]);

                    let path = block.source_file.to_owned().unwrap_or_default();
                    let name = if block.is_unnamed {
                        ""
                    } else {
                        block.name.as_ref().map(|n| &n[..]).unwrap_or("")
                    };

                    if !clean {
                        result.push(format!(
                            "{}{} {}{}#{}{}",
                            &self.indent,
                            comment_start,
                            if idx == 0 { &block_start } else { &block_next },
                            path,
                            name,
                            comment_end,
                        ));
                    }
                    result.push(
                        block
                            .compile_with(code_blocks, scope, settings)
                            .map(|code| {
                                code.split('\n')
                                    .map(|line| {
                                        if blank_lines && line.trim().is_empty() {
                                            "".to_string()
                                        } else {
                                            format!("{}{}", self.indent, line)
                                        }
                                    })
                                    .collect::<Vec<_>>()
                                    .join("\n")
                            })?,
                    );

                    if !clean && idx == blocks.len() - 1 {
                        result.push(format!(
                            "{}{} {}{}#{}{}",
                            &self.indent, comment_start, &block_end, path, name, comment_end,
                        ));
                    }
                }
                Ok(result.join("\n"))
            }
        }
    }
}
