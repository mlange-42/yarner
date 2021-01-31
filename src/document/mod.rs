//! The internal representation of a literate document
pub mod code;
pub mod text;
pub mod transclusion;

use self::code::CodeBlock;
use self::text::TextBlock;
use self::transclusion::Transclusion;

use crate::config::LanguageSettings;
use crate::parser::{code::RevCodeBlock, md::MdParser};
use std::collections::hash_map::{Entry, HashMap};
use std::fmt::Write;

/// A representation of a `Document` of literate code
#[derive(Debug)]
pub struct Document {
    nodes: Vec<Node>,
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

    /// Formats this `Document` as a string containing the documentation file contents
    pub fn print_docs(&self, printer: &MdParser) -> String {
        let mut output = String::new();
        for node in &self.nodes {
            match node {
                Node::Transclusion(transclusion) => {
                    output.push_str(&printer.print_transclusion(transclusion, false))
                }
                Node::Text(text_block) => output.push_str(&printer.print_text_block(text_block)),
                Node::Code(code_block) => {
                    if !code_block.hidden {
                        output.push_str(
                            &printer
                                .print_code_block(code_block)
                                .split('\n')
                                .map(|line| {
                                    if line.is_empty() {
                                        line.to_string()
                                    } else {
                                        format!("{}{}", code_block.indent, line)
                                    }
                                })
                                .collect::<Vec<_>>()
                                .join("\n"),
                        )
                    }
                }
            }
        }
        output
    }

    /// Formats this `Document` as the original source, potentially replacing code blocks
    pub fn print_reverse(
        &self,
        printer: &MdParser,
        code_blocks: &HashMap<&Option<String>, &Vec<RevCodeBlock>>,
    ) -> String {
        let mut block_count: HashMap<&Option<String>, usize> = HashMap::new();

        let mut output = String::new();
        for node in &self.nodes {
            match node {
                Node::Transclusion(transclusion) => {
                    output.push_str(&printer.print_transclusion(transclusion, true))
                }
                Node::Text(text_block) => output.push_str(&printer.print_text_block(text_block)),
                Node::Code(code_block) => {
                    let index = match block_count.entry(&code_block.name) {
                        Entry::Occupied(mut entry) => {
                            let old_index = *entry.get();
                            entry.insert(old_index + 1)
                        }
                        Entry::Vacant(entry) => {
                            entry.insert(1);
                            0
                        }
                    };
                    let alt_block = if let Some(blocks) = code_blocks.get(&code_block.name) {
                        blocks.get(index)
                    } else {
                        None
                    };
                    output.push_str(
                        &printer
                            .print_code_block_reverse(code_block, alt_block)
                            .split('\n')
                            .map(|line| {
                                if line.is_empty() {
                                    line.to_string()
                                } else {
                                    format!("{}{}", code_block.indent, line)
                                }
                            })
                            .collect::<Vec<_>>()
                            .join("\n"),
                    )
                }
            }
        }
        output
    }

    /// Formats this `Document` as a string containing the compiled code
    pub fn print_code(
        &self,
        entrypoint: Option<&str>,
        language: Option<&str>,
        settings: Option<&LanguageSettings>,
    ) -> Result<String, CompileError> {
        let comment_start = settings
            .and_then(|s| s.block_labels.as_ref())
            .map(|l| &l.comment_start[..])
            .unwrap_or("");
        let comment_end = settings
            .and_then(|s| s.block_labels.as_ref())
            .and_then(|l| l.comment_end.as_ref().map(|e| &e[..]))
            .unwrap_or("");
        let block_start = settings
            .and_then(|s| s.block_labels.as_ref())
            .map(|l| &l.block_start[..])
            .unwrap_or("");
        let block_end = settings
            .and_then(|s| s.block_labels.as_ref())
            .map(|l| &l.block_end[..])
            .unwrap_or("");
        let block_next = settings
            .and_then(|s| s.block_labels.as_ref())
            .map(|l| &l.block_next[..])
            .unwrap_or("");
        let block_name_sep = settings
            .and_then(|s| s.block_labels.as_ref())
            .map(|l| &l.name_separator[..])
            .unwrap_or("");

        let clean = if let Some(s) = settings {
            s.clean_code || s.block_labels.is_none()
        } else {
            true
        };

        let code_blocks = self.code_blocks_by_name(language);
        let mut result = String::new();
        match code_blocks.get(&entrypoint) {
            Some(blocks) => {
                for (idx, block) in blocks.iter().enumerate() {
                    let path = block.source_file.to_owned().unwrap_or_default();
                    let name = if block.is_unnamed {
                        ""
                    } else {
                        block.name.as_ref().map(|n| &n[..]).unwrap_or("")
                    };

                    if !clean {
                        let sep = if idx == 0 || block.name != blocks[idx - 1].name {
                            &block_start
                        } else {
                            &block_next
                        };
                        writeln!(
                            result,
                            "{} {}{}{}{}{}",
                            comment_start, sep, path, block_name_sep, name, comment_end,
                        )
                        .unwrap();
                    }
                    result.push_str(&block.compile(&code_blocks, settings)?);
                    result.push('\n');
                    if !clean && (idx == blocks.len() - 1 || block.name != blocks[idx + 1].name) {
                        write!(
                            result,
                            "{} {}{}{}{}{}",
                            comment_start, block_end, path, block_name_sep, name, comment_end,
                        )
                        .unwrap();
                    }
                }
            }
            None => {
                return Err(CompileError::Single {
                    line_number: 0,
                    kind: CompileErrorKind::MissingEntrypoint,
                })
            }
        }
        if settings.map(|s| s.eof_newline).unwrap_or(true) && !result.ends_with('\n') {
            result.push('\n');
        }
        Ok(result)
    }

    /// Return the nodes of the document
    pub fn nodes(&self) -> &[Node] {
        &self.nodes
    }

    /// Return the nodes of the document in mutable form
    pub fn nodes_mut(&mut self) -> &mut [Node] {
        &mut self.nodes
    }

    /// Return the nodes of the document, consuming the document
    pub fn into_nodes(self) -> Vec<Node> {
        self.nodes
    }

    /// Gets all the text blocks of this document
    pub fn text_blocks_mut(&mut self) -> impl Iterator<Item = &mut TextBlock> {
        self.nodes.iter_mut().filter_map(|node| match node {
            Node::Text(block) => Some(block),
            _ => None,
        })
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

    fn code_blocks_by_name<'a>(
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
}

/// Problems encountered while compiling the document
#[derive(Debug)]
pub enum CompileErrorKind {
    /// An unknown meta variable was encountered
    UnknownMetaVariable(String),
    /// An unknown macro name was encountered
    UnknownMacro(String),
    /// Meta variables incorrect
    InvalidVariables(String),
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
