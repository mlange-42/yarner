//! The internal representation of a literate document
use std::iter::FromIterator;

pub mod ast;
pub mod code;
pub mod text;
pub mod tranclusion;

use self::ast::Ast;
use crate::config::LanguageSettings;
use crate::parser::{code::RevCodeBlock, md::MdParser};
use std::collections::HashMap;

/// A representation of a `Document` of literate code
#[derive(Debug)]
pub struct Document {
    tree: Ast,
}

impl Document {
    /// Creates a new document with the tree
    pub(crate) fn new(tree: Ast) -> Self {
        Document { tree }
    }

    /// Formats this `Document` as a string containing the documentation file contents
    pub fn print_docs(&self, printer: &MdParser) -> String {
        self.tree.print_docs(printer)
    }

    /// Formats this `Document` as the original source, potentially replacing code blocks
    pub fn print_reverse(
        &self,
        printer: &MdParser,
        code_blocks: &HashMap<&Option<String>, &Vec<RevCodeBlock>>,
    ) -> String {
        self.tree.print_reverse(printer, code_blocks)
    }

    /// Formats this `Document` as a string containing the compiled code
    pub fn print_code(
        &self,
        entrypoint: &Option<&str>,
        language: &Option<&str>,
        settings: &Option<&LanguageSettings>,
    ) -> Result<String, CompileError> {
        self.tree.print_code(entrypoint, language, settings)
    }

    /// Return the document's AST
    pub fn tree(&self) -> &Ast {
        &self.tree
    }

    /// Return the document's AST in mutable form
    pub fn tree_mut(&mut self) -> &mut Ast {
        &mut self.tree
    }

    /// Return the document's AST, consuming the document
    pub fn into_tree(self) -> Ast {
        self.tree
    }
}

impl<T> FromIterator<T> for Document
where
    Ast: FromIterator<T>,
{
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self::new(iter.into_iter().collect())
    }
}

impl<T> From<T> for Document
where
    Ast: From<T>,
{
    fn from(value: T) -> Self {
        Self::new(value.into())
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

impl FromIterator<CompileError> for CompileError {
    fn from_iter<I: IntoIterator<Item = CompileError>>(iter: I) -> Self {
        CompileError::Multi(iter.into_iter().collect())
    }
}
