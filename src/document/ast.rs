//! Internal AST representation

use std::collections::HashMap;
use std::iter::FromIterator;

use super::code::CodeBlock;
use super::text::TextBlock;
use super::{CompileError, CompileErrorKind};
use crate::document::tranclusion::Transclusion;
use crate::parser::Printer;

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

    /// Gets all the code blocks of this AST, concatenating blocks of the same name
    pub(crate) fn code_blocks(&self, language: Option<&str>) -> HashMap<Option<&str>, CodeBlock> {
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
                    .and_modify(|existing: &mut CodeBlock| existing.append(block))
                    .or_insert_with(|| block.clone());
            }
        }
        code_blocks
    }
    /// Gets all the text blocks of this AST
    pub(crate) fn text_blocks(&self) -> Vec<&TextBlock> {
        self.nodes
            .iter()
            .filter_map(|node| match node {
                Node::Text(block) => Some(block),
                _ => None,
            })
            .collect()
    }

    /// Gets all the transclusions of this AST
    pub fn transclusions(&self) -> Vec<Transclusion> {
        let vec = self
            .nodes
            .iter()
            .filter_map(|node| match node {
                Node::Transclusion(trans) => Some((*trans).clone()),
                _ => None,
            })
            .collect();
        vec
    }

    /// Renders the program this AST is representing in the documentation format
    pub(crate) fn print_docs<P: Printer>(&self, printer: &P) -> String {
        let mut output = String::new();
        for node in &self.nodes {
            match node {
                Node::Transclusion(transclusion) => {
                    output.push_str(&printer.print_transclusion(transclusion))
                }
                Node::Text(text_block) => output.push_str(&printer.print_text_block(text_block)),
                Node::Code(code_block) => {
                    if !code_block.hidden {
                        output.push_str(
                            &printer
                                .print_code_block(code_block)
                                .split("\n")
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

    /// Renders the program this AST is representing in the code format
    pub(crate) fn print_code(
        &self,
        entrypoint: Option<&str>,
        language: Option<&str>,
    ) -> Result<String, CompileError> {
        let code_blocks = self.code_blocks(language);
        code_blocks
            .get(&entrypoint)
            .map(|entrypoint| entrypoint.compile(&code_blocks))
            .unwrap_or(Err(CompileError::Single {
                line_number: 0,
                kind: CompileErrorKind::MissingEntrypoint,
            }))
    }

    /// Renders the program this AST is representing in the documentation format
    pub fn transclude(&mut self, replace: &Transclusion, with: Ast) {
        let mut index = 0;
        while index < self.nodes.len() {
            if let Node::Transclusion(trans) = &self.nodes[index] {
                if trans == replace {
                    for (i, node) in with.nodes.into_iter().enumerate() {
                        self.nodes.insert(index + i, node);
                    }
                    // TODO: currently, only one transclusion of a particular document is possible.
                    // May be sufficient, but should be checked.
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
