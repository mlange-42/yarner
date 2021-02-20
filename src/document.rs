//! The internal representation of a literate document
use crate::config::ParserSettings;

use std::collections::HashMap;
use std::path::{Path, PathBuf};

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

    pub fn transclude(&mut self, replace: &Transclusion, with: Document, from: &str) {
        let mut index = 0;
        while index < self.nodes.len() {
            if let Node::Transclusion(trans) = &self.nodes[index] {
                if trans == replace {
                    self.nodes.remove(index);
                    for (i, mut node) in with.nodes.into_iter().enumerate() {
                        if let Node::Code(code) = &mut node {
                            if code.name.is_none() {
                                code.name = Some(from.to_string());
                                code.is_unnamed = true;
                            }
                            if code.source_file.is_none() {
                                code.source_file = Some(replace.file.to_str().unwrap().to_owned());
                            }
                        };
                        self.nodes.insert(index + i, node);
                    }
                    // TODO: currently, only a single transclusion of a particular document is possible.
                    // May be sufficient (or even desired), but should be checked.
                    break;
                }
            }
            index += 1;
        }
    }

    /// Finds all file-specific entry points
    pub fn entry_points(
        &self,
        settings: &ParserSettings,
    ) -> HashMap<Option<&str>, (&Path, Option<PathBuf>)> {
        let mut entries = HashMap::new();
        let pref = &settings.file_prefix;
        for block in self.code_blocks() {
            if let Some(name) = block.name.as_deref() {
                if let Some(rest) = name.strip_prefix(pref) {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Config, LF_NEWLINE};
    use crate::document::Node::{Code, Text};

    #[test]
    fn document_elements() {
        let mut config = toml::from_str::<Config>(include_str!("create/Yarner.toml")).unwrap();
        config.paths.entrypoint = Some("Main".to_owned());

        let mut code = CodeBlock::new();
        code.name = Some("file:README.md#Main".to_owned());
        let doc = Document {
            nodes: vec![
                Text(TextBlock { text: vec![] }),
                Code(code),
                Text(TextBlock { text: vec![] }),
                Node::Transclusion(Transclusion::new(
                    PathBuf::from("trans.md"),
                    "@{{trans.md}}".to_string(),
                )),
            ],
            newline: LF_NEWLINE,
        };

        let code_blocks: Vec<_> = doc.code_blocks().collect();
        assert_eq!(code_blocks.len(), 1);

        let transclusions: Vec<_> = doc.transclusions().collect();
        assert_eq!(transclusions.len(), 1);

        let entrypoints = doc.entry_points(&config.parser);
        assert_eq!(entrypoints.len(), 1);

        let entry = &entrypoints[&Some("file:README.md#Main")];
        assert_eq!(entry.0.to_str(), Some("README.md#Main"));
        assert_eq!(entry.1, None);
    }
}
