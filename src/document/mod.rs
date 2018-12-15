//! The internal representation of a literate document
use std::iter::FromIterator;

pub mod ast;
pub mod code;
pub mod text;

use crate::parser::Printer;
use self::ast::Ast;

/// A representation of a `Document` of literate code
#[derive(Debug)]
pub struct Document<'a> {
    tree: Ast<'a>,
}

impl<'a> Document<'a> {
    /// Creates a new document with the tree
    pub(crate) fn new(tree: Ast<'a>) -> Self {
        Document { tree }
    }

    /// Formats this [`Document`] as a string containing the documentation file contents
    pub fn print_docs<P: Printer>(&self, printer: &P) -> String {
        self.tree.print_docs(printer)
    }

    /// Formats this [`Document`] as a string containing the compiled code
    pub fn print_code(&self, entrypoint: Option<&str>, language: Option<&str>) -> Result<String, CompileError> {
        self.tree.print_code(entrypoint, language)
    }
}

impl<'a, T> FromIterator<T> for Document<'a>
where Ast<'a>: FromIterator<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self::new(iter.into_iter().collect())
    }
}

impl<'a, T> From<T> for Document<'a>
where Ast<'a>: From<T> {
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
    }
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
            CompileError::Single { line_number, kind } => writeln!(f, "{:?} (line {})", kind, line_number),
        }
    }
}

impl std::error::Error for CompileError {}

impl FromIterator<CompileError> for CompileError {
    fn from_iter<I: IntoIterator<Item = CompileError>>(iter: I) -> Self {
        CompileError::Multi(iter.into_iter().collect())
    }
}
