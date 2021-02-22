//! The internal representation of a literate document
use std::collections::HashMap;
use std::path::PathBuf;

/// A representation of a `Document` of literate code
#[derive(Debug)]
pub struct Document {
    pub nodes: Vec<Node>,
    newline: &'static str,
}

#[derive(Debug)]
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
    pub fn new(nodes: Vec<Node>, newline: &'static str) -> Self {
        Document { nodes, newline }
    }

    pub fn newline(&self) -> &str {
        self.newline
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
}

/// A `TextBlock` is just text that will be copied verbatim into the output documentation file
#[derive(Debug, Default)]
pub struct TextBlock {
    /// The source text
    text: Vec<String>,
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
#[derive(Debug, PartialEq, Clone)]
pub struct Transclusion {
    /// The target file path
    pub file: PathBuf,
    /// The original string of the transclusion
    pub original: String,
}

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
    pub fn line_number(&self) -> Option<usize> {
        self.source.first().map(|line| line.line_number)
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
