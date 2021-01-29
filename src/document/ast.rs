//! Internal AST representation

use std::collections::HashMap;
use std::iter::FromIterator;

use super::code::CodeBlock;
use super::text::TextBlock;
use super::{CompileError, CompileErrorKind};
use crate::config::LanguageSettings;
use crate::document::tranclusion::Transclusion;
use crate::parser::code::RevCodeBlock;
use crate::parser::md::MdParser;
use std::collections::hash_map::Entry;

/// A `Node` in the `Ast`
#[derive(Debug)]
pub(crate) enum Node {
    /// A text block
    Text(TextBlock),
    /// A code block
    Code(CodeBlock),
    /// A transclusion
    Transclusion(Transclusion),
}

/// The AST of a literate document
#[derive(Debug)]
pub struct Ast {
    /// A list of the nodes in this "tree", which is actually just a sequence until the parsers get
    /// cooler.
    nodes: Vec<Node>,
}

impl Ast {
    /// Create a new empty AST
    pub(crate) fn new(nodes: Vec<Node>) -> Self {
        Ast { nodes }
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

    /// Gets all the code blocks of this AST, concatenating blocks of the same name
    pub(crate) fn code_blocks(
        &self,
        language: &Option<&str>,
    ) -> HashMap<Option<&str>, Vec<CodeBlock>> {
        let mut code_blocks = HashMap::new();
        for node in &self.nodes {
            if let Node::Code(block) = node {
                // skip blocks in the wrong language. If either language is None, then assume it is
                // ok
                if let Some(language) = language {
                    if let Some(block_language) = &block.language {
                        if language != block_language {
                            continue;
                        }
                    }
                }
                code_blocks
                    .entry(block.name.as_ref().map(|x| &x[..])) // TODO: any nicer way to write this
                    .and_modify(|existing: &mut Vec<CodeBlock>| existing.push(block.clone()))
                    .or_insert_with(|| vec![block.clone()]);
            }
        }
        code_blocks
    }
    /// Gets all the text blocks of this AST
    #[allow(dead_code)]
    pub(crate) fn text_blocks(&self) -> Vec<&TextBlock> {
        self.nodes
            .iter()
            .filter_map(|node| match node {
                Node::Text(block) => Some(block),
                _ => None,
            })
            .collect()
    }

    /// Gets all the text blocks of this AST
    pub(crate) fn text_blocks_mut(&mut self) -> Vec<&mut TextBlock> {
        self.nodes
            .iter_mut()
            .filter_map(|node| match node {
                Node::Text(block) => Some(block),
                _ => None,
            })
            .collect()
    }

    /// Gets all the transclusions of this AST
    pub fn transclusions(&self) -> Vec<Transclusion> {
        self.nodes
            .iter()
            .filter_map(|node| match node {
                Node::Transclusion(trans) => Some((*trans).clone()),
                _ => None,
            })
            .collect()
    }

    /// Renders the program this AST is representing in the documentation format
    pub(crate) fn print_docs(&self, printer: &MdParser) -> String {
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

    /// Renders the program this AST is representing back into the original format, replacing code blocks
    pub(crate) fn print_reverse(
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

    /// Renders the program this AST is representing in the code format
    pub(crate) fn print_code(
        &self,
        entrypoint: &Option<&str>,
        language: &Option<&str>,
        settings: &Option<&LanguageSettings>,
    ) -> Result<String, CompileError> {
        let comment_start = settings.map(|s| &s.comment_start[..]).unwrap_or("");
        let comment_end = settings
            .and_then(|s| s.comment_end.as_ref().map(|e| &e[..]))
            .unwrap_or("");
        let block_start = settings.map(|s| &s.block_start[..]).unwrap_or("");
        let block_end = settings.map(|s| &s.block_end[..]).unwrap_or("");

        let clean = if let Some(s) = settings {
            s.clean_code
        } else {
            true
        };

        let code_blocks = self.code_blocks(&language);
        let mut result = String::new();
        match code_blocks.get(&entrypoint) {
            Some(blocks) => {
                for block in blocks {
                    let path = block.source_file.to_owned().unwrap_or_default();
                    let name = if block.is_unnamed {
                        ""
                    } else {
                        block.name.as_ref().map(|n| &n[..]).unwrap_or("")
                    };

                    if !clean {
                        result.push_str(&format!(
                            "{} {}{}#{}{}\n",
                            comment_start, block_start, path, name, comment_end,
                        ));
                    }
                    result.push_str(&block.compile(&code_blocks, settings)?);
                    result.push('\n');
                    if !clean {
                        result.push_str(&format!(
                            "{} {}{}#{}{}",
                            comment_start, block_end, path, name, comment_end,
                        ));
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

    /// Transclusion
    pub fn transclude(&mut self, replace: &Transclusion, with: Ast, from_source: &str, from: &str) {
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

impl FromIterator<Node> for Ast {
    fn from_iter<I: IntoIterator<Item = Node>>(iter: I) -> Self {
        Self::new(iter.into_iter().collect())
    }
}

impl From<Vec<Node>> for Ast {
    fn from(nodes: Vec<Node>) -> Self {
        Self::new(nodes)
    }
}
