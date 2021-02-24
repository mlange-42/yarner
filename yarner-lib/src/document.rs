//! The internal representation of a literate document
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::default::Default;
use std::path::PathBuf;

/// A representation of a `Document` of literate code
#[derive(Debug, Serialize, Deserialize)]
pub struct Document {
    /// The nodes forming the document
    pub nodes: Vec<Node>,
    /// The newline character(s) used in the sources
    pub newline: String,
}

/// A node, representing text and code blocks, as well as transclusions
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

    /// The newline character(s) used in the sources
    pub fn newline(&self) -> &str {
        &self.newline
    }

    /// Gets all the code blocks of this document
    pub fn code_blocks(&self) -> impl Iterator<Item = &CodeBlock> {
        self.nodes.iter().filter_map(|node| match node {
            Node::Code(block) => Some(block),
            _ => None,
        })
    }

    /// Code blocks, mapped by name
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
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct TextBlock {
    /// The source text
    pub text: Vec<String>,
}

/// A `Transclusion` is a reference to another file that should be pulled into the source
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Transclusion {
    /// The target file path
    pub file: PathBuf,
    /// The original string of the transclusion
    pub original: String,
}

/// A `CodeBlock` is a block of code as defined by the input format.
#[derive(Default, Debug, Serialize, Deserialize)]
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
    pub fn new(
        line_number: usize,
        indent: String,
        language: Option<String>,
        alternative: bool,
    ) -> Self {
        CodeBlock {
            line_number,
            indent,
            language,
            alternative,
            ..Default::default()
        }
    }
}

/// A `Source` represents the source code on a line.
#[derive(Debug, Serialize, Deserialize)]
pub enum Line {
    /// A macro invocation
    Macro {
        /// Indentation of the line, without block indent
        indent: String,
        /// Name of the macro
        name: String,
    },
    /// A line of source code
    Source {
        /// Indentation of the line, without block indent
        indent: String,
        /// Source code in the line
        source: String,
    },
}
