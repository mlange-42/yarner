//! The internal representation of a literate document
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// A representation of a `Document` of literate code
#[derive(Debug, Serialize, Deserialize)]
pub struct Document {
    pub nodes: Vec<Node>,
    newline: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Node {
    /// A text block
    Text(TextBlock),
    /// A code block
    Code(CodeBlock),
    /// A transclusion
    Transclusion(Transclusion),
}

impl Document {
    /// Creates a new document with the given nodes
    pub fn new(nodes: Vec<Node>, newline: String) -> Self {
        Document { nodes, newline }
    }

    pub fn newline(&self) -> &str {
        &self.newline
    }

    /// Sets the source file for all code blocks that have none
    pub fn set_source(&mut self, source: &str) {
        for node in &mut self.nodes {
            if let Node::Code(block) = node {
                if block.source_file.is_none() {
                    block.source_file = Some(source.to_owned());
                }
            }
        }
    }

    /// Gets all the code blocks of this document
    pub fn code_blocks(&self) -> impl Iterator<Item = &CodeBlock> {
        self.nodes.iter().filter_map(|node| match node {
            Node::Code(block) => Some(block),
            _ => None,
        })
    }

    pub fn code_blocks_by_name(&self) -> HashMap<Option<&str>, Vec<&CodeBlock>> {
        let mut code_blocks = HashMap::<_, Vec<&CodeBlock>>::new();

        for block in self.code_blocks() {
            code_blocks
                .entry(block.name.as_deref())
                .or_default()
                .push(block);
        }

        code_blocks
    }

    /// Gets all the transclusions of this document
    pub fn transclusions(&self) -> impl Iterator<Item = &Transclusion> {
        self.nodes.iter().filter_map(|node| match node {
            Node::Transclusion(trans) => Some(trans),
            _ => None,
        })
    }

    /// Finds all file-specific entry points
    pub fn entry_points(
        &self,
        file_prefix: &str,
    ) -> HashMap<Option<&str>, (&Path, Option<PathBuf>)> {
        let mut entries = HashMap::new();
        for block in self.code_blocks() {
            if let Some(name) = block.name.as_deref() {
                if let Some(rest) = name.strip_prefix(file_prefix) {
                    entries.insert(
                        Some(name),
                        (
                            Path::new(rest),
                            block.source_file.as_ref().map(|file| file.into()),
                        ),
                    );
                }
            }
        }
        entries
    }
}

/// A `TextBlock` is just text that will be copied verbatim into the output documentation file
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct TextBlock {
    /// The source text
    pub text: Vec<String>,
}

impl TextBlock {
    /// Creates a new empty `TextBlock`
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a line to this `TextBlock`
    pub fn add_line(&mut self, line: String) {
        self.text.push(line);
    }

    pub fn lines(&self) -> &[String] {
        &self.text
    }
}

/// A `Transclusion` is a reference to another file that should be pulled into the source
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Transclusion {
    /// The target file path
    file: PathBuf,
    /// The original string of the transclusion
    original: String,
}

impl Transclusion {
    /// Creates a new `Transclusion`
    pub fn new(file: PathBuf, original: String) -> Self {
        Transclusion { file, original }
    }

    /// The path to the file this transclusion points to
    pub fn file(&self) -> &PathBuf {
        &self.file
    }

    /// The original string of the transclusion
    pub fn original(&self) -> &str {
        &self.original
    }
}

/// A `CodeBlock` is a block of code as defined by the input format.
#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct CodeBlock {
    /// Source line number of the first code line
    pub line_number: usize,
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

    /// Returns the line number of the first line in this code block
    pub fn line_number(&self) -> usize {
        self.line_number
    }
}

/// A `Source` represents the source code on a line.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Source {
    /// A macro invocation, resolved by the literate compiler
    Macro(String),
    /// Source text, possibly including meta variable interpolations
    Source(String),
}

/// A `Line` defines a line of code.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Line {
    /// The indent on this line. An indent is defined as leading whitespace (spaces/tabs)
    pub indent: String,
    /// The source text
    pub source: Source,
    /// The literate compiler defined comment - this is extracted from source text to be rendered
    /// in an appropriate format in the documentation, rather than as a comment in the source file
    pub comment: Option<String>,
}
