//! Internal AST representation

use std::iter::FromIterator;
use std::collections::HashMap;

use super::text::TextBlock;
use super::code::CodeBlock;
use super::{CompileError, CompileErrorKind};
use crate::parser::Printer;

/// A [`Node`] in the [`Ast`]
#[derive(Debug)]
pub(crate) enum Node<'a> {
    /// A text block
    Text(TextBlock<'a>),
    /// A code block
    Code(CodeBlock<'a>),
}

/// The AST of a literate document
#[derive(Debug)]
pub struct Ast<'a> {
    /// A list of the nodes in this "tree", which is actually just a sequence until the parsers get
    /// cooler.
    nodes: Vec<Node<'a>>,
}

impl<'a> Ast<'a> {
    /// Create a new empty AST
    pub(crate) fn new(nodes: Vec<Node<'a>>) -> Self {
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
                    .and_modify(|existing: &mut CodeBlock<'a>| existing.append(block))
                    .or_insert_with(|| block.clone());
            }
        }
        code_blocks
    }

    /// Renders the program this AST is representing in the documentation format
    pub(crate) fn print_docs<P: Printer>(&self, printer: &P) -> String {
        let mut output = String::new();
        for node in &self.nodes {
            match node {
                Node::Text(text_block) => output.push_str(&printer.print_text_block(text_block)),
                Node::Code(code_block) => output.push_str(&printer
                    .print_code_block(code_block)
                    .split("\n")
                    .map(|line| if line.is_empty() { line.to_string() } else { format!("{}{}", code_block.indent, line) })
                    .collect::<Vec<_>>()
                    .join("\n")
                ),
            }
        }
        output
    }

    /// Renders the program this AST is representing in the code format
    pub(crate) fn print_code(&self, entrypoint: Option<&str>, language: Option<&str>) -> Result<String, CompileError> {
        let code_blocks = self.code_blocks(language);
        code_blocks
            .get(&entrypoint)
            .map(|entrypoint| entrypoint.compile(&code_blocks))
            .unwrap_or(Err(CompileError::Single { line_number: 0, kind: CompileErrorKind::MissingEntrypoint }))
    }
}

impl<'a> FromIterator<Node<'a>> for Ast<'a>  {
    fn from_iter<I: IntoIterator<Item = Node<'a>>>(iter: I) -> Self {
        Self::new(iter.into_iter().collect())
    }
}

impl<'a> From<Vec<Node<'a>>> for Ast<'a> {
    fn from(nodes: Vec<Node<'a>>) -> Self {
        Self::new(nodes)
    }
}
