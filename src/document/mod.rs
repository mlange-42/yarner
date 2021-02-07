//! The internal representation of a literate document
pub mod code;
pub mod text;
pub mod transclusion;

use self::code::CodeBlock;
use self::text::TextBlock;
use self::transclusion::Transclusion;

use crate::config::ParserSettings;
use std::collections::hash_map::HashMap;
use std::path::{Path, PathBuf};

/// A representation of a `Document` of literate code
#[derive(Debug)]
pub struct Document {
    pub nodes: Vec<Node>,
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
    pub fn new(nodes: Vec<Node>) -> Self {
        Document { nodes }
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
    pub fn code_blocks<'a>(
        &'a self,
        language: Option<&'a str>,
    ) -> impl Iterator<Item = &'a CodeBlock> {
        self.nodes.iter().filter_map(move |node| match node {
            Node::Code(block) => {
                if let (Some(lhs), Some(rhs)) = (language, &block.language) {
                    if lhs != rhs {
                        return None;
                    }
                }

                Some(block)
            }
            _ => None,
        })
    }

    pub fn code_blocks_by_name<'a>(
        &'a self,
        language: Option<&'a str>,
    ) -> HashMap<Option<&'a str>, Vec<&'a CodeBlock>> {
        let mut code_blocks = HashMap::<_, Vec<&CodeBlock>>::new();

        for block in self.code_blocks(language) {
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

    pub fn transclude(
        &mut self,
        replace: &Transclusion,
        with: Document,
        from_source: &str,
        from: &str,
    ) {
        let mut index = 0;
        while index < self.nodes.len() {
            if let Node::Transclusion(trans) = &self.nodes[index] {
                if trans == replace {
                    self.nodes.remove(index);
                    for (i, mut node) in with.nodes.into_iter().enumerate() {
                        if let Node::Code(code) = &mut node {
                            // TODO use entrypoint option here, too? Currently, only in main file.
                            if code.name.is_none() {
                                code.name = Some(from.to_string());
                                code.is_unnamed = true;
                            }
                            // TODO: move to parser?
                            if code.source_file.is_none() {
                                code.source_file = Some(from_source.to_string());
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
    pub fn entry_points<'a>(
        &'a self,
        settings: &ParserSettings,
        language: Option<&'a str>,
    ) -> HashMap<Option<&'a str>, (&'a Path, Option<PathBuf>)> {
        let mut entries = HashMap::new();
        let pref = &settings.file_prefix;
        for block in self.code_blocks(language) {
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

/// Problems encountered while compiling the document
#[derive(Debug)]
pub enum CompileErrorKind {
    /// An unknown macro name was encountered
    UnknownMacro(String),
    /// There is no unnamed code block to use as the entrypoint
    MissingEntrypoint,
}

/// Errors that were encountered while compiling the document
#[derive(Debug)]
pub enum CompileError {
    #[doc(hidden)]
    Multi(Vec<CompileError>),
    #[doc(hidden)]
    Single {
        line_number: usize,
        kind: CompileErrorKind,
    },
}

impl std::fmt::Display for CompileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompileError::Multi(errors) => {
                for error in errors {
                    writeln!(f, "{}", error)?;
                }
                Ok(())
            }
            CompileError::Single { line_number, kind } => {
                writeln!(f, "{:?} (line {})", kind, line_number)
            }
        }
    }
}

impl std::error::Error for CompileError {}
