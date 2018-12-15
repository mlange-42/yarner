//! Representation of the code parts of the AST

use std::collections::HashMap;
use super::{CompileError, CompileErrorKind};
use crate::util::try_collect::TryCollectExt;

/// A `Segment` is some of the raw source text.
#[derive(Clone, Debug)]
pub enum Segment<'a> {
    /// Raw source text
    Source(&'a str),
    /// A meta variable, to be interpolated by the literate compiler
    MetaVar(&'a str),
}

/// A `Source` represents the source code on a line.
#[derive(Clone, Debug)]
pub enum Source<'a> {
    /// A macro invocation, resolved by the literate compiler
    Macro {
        /// The name of the macro
        name: String,
        /// The meta-variable values to interpolate
        scope: Vec<&'a str>
    },
    /// Source text, possibly including meta variable interpolations
    Source(Vec<Segment<'a>>)
}

/// A `Line` defines a line of code.
#[derive(Clone, Debug)]
pub struct Line<'a> {
    /// The line number of this line (original source)
    pub line_number: usize,
    /// The indent on this line. An indent is defined as leading whitespace (spaces/tabs)
    pub indent: &'a str,
    /// The source text
    pub source: Source<'a>,
    /// The literate compiler defined comment - this is extracted from source text to be rendered
    /// in an appropriate format in the documentation, rather than as a comment in the source file
    pub comment: Option<&'a str>,
}

impl<'a> Line<'a> {
    fn compile_with(&self, code_blocks: &HashMap<Option<&str>, CodeBlock<'a>>, scope: &HashMap<&str, &str>) -> Result<String, CompileError> {
        match &self.source {
            Source::Source(segments) => {
                let code = segments
                    .iter()
                    .map(|segment| match segment {
                        Segment::Source(source) => Ok(*source),
                        Segment::MetaVar(name) => scope.get(name)
                            .map(|var| *var)
                            .ok_or(CompileError::Single {
                                line_number: self.line_number,
                                kind: CompileErrorKind::UnknownMetaVariable(name.to_string()),
                            }),
                    })
                    .try_collect()
                    .map(|vec: Vec<_>| vec.join(""))?;
                Ok(format!("{}{}", self.indent, code))
            },
            Source::Macro { name, scope } => code_blocks.get(&Some(name))
                .ok_or(CompileError::Single {
                    line_number: self.line_number,
                    kind: CompileErrorKind::UnknownMacro(name.to_string()),
                })
                .and_then(|block| block
                    .compile_with(code_blocks, block.assign_vars(scope))
                    .map(|code| code
                        .split("\n")
                        .map(|line| format!("{}{}", self.indent, line))
                        .collect::<Vec<_>>()
                        .join("\n")
                    )
                ),
        }
    }
}

/// A `CodeBlock` is a block of code as defined by the input format.
#[derive(Clone, Default, Debug)]
pub struct CodeBlock<'a> {
    /// The indent of this code block is in the documentation file
    pub indent: &'a str,
    /// The name of this code block
    pub name: Option<String>,
    /// The variables extracted from the name
    pub vars: Vec<&'a str>,
    /// The language this block was written in
    pub language: Option<String>,
    /// The source is the lines of code
    pub source: Vec<Line<'a>>,
}

impl<'a> CodeBlock<'a> {
    /// Creates a new empty `CodeBlock`
    pub fn new() -> Self { Self::default() }

    /// Indents this code block
    pub fn indented(self, indent: &'a str) -> Self {
        Self { indent, ..self }
    }

    /// Names this code block
    pub fn named(self, name: String, vars: Vec<&'a str>) -> Self {
        Self { name: Some(name), vars, ..self }
    }

    /// Sets the language of this code block
    pub fn in_language(self, language: String) -> Self {
        Self { language: Some(language), ..self }
    }

    /// Adds a line to this code block
    pub fn add_line(&mut self, line: Line<'a>) {
        self.source.push(line);
    }

    /// Appends another code block to the end of this one
    pub fn append(&mut self, other: &CodeBlock<'a>) {
        self.source.extend_from_slice(&other.source)
    }

    /// "Compiles" this code block into its output code
    pub fn compile(&self, code_blocks: &HashMap<Option<&str>, CodeBlock<'a>>) -> Result<String, CompileError> {
        self.compile_with(code_blocks, HashMap::default())
    }

    /// Returns the line number of the first line in this code block
    pub fn line_number(&self) -> Option<usize> {
        self.source.first().map(|line| line.line_number)
    }

    fn compile_with(&self, code_blocks: &HashMap<Option<&str>, CodeBlock<'a>>, scope: HashMap<&str, &str>) -> Result<String, CompileError> {
        self.source
            .iter()
            .map(|line| line.compile_with(code_blocks, &scope))
            .try_collect()
            .map(|vec: Vec<_>| vec.join("\n"))
    }

    fn assign_vars(&self, scope: &[&'a str]) -> HashMap<&str, &'a str> {
        self.vars
            .iter()
            .zip(scope)
            .map(|(name, value)| (*name, *value))
            .collect()
    }
}
