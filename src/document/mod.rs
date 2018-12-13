//! The internal representation of a literate document
use std::iter::FromIterator;
use std::collections::HashMap;

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
    pub fn new(tree: Ast<'a>) -> Self {
        Document { tree }
    }

    /// Formats this [`Document`] as a string containing the documentation file contents
    pub fn print_docs<P: Printer>(&self, printer: &P) -> String {
        self.tree.print_docs(printer)
    }

    /// Formats this [`Document`] as a string containing the compiled code
    pub fn print_code(&self) -> Result<String, CompileError> {
        self.tree.print_code()
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

#[derive(Debug)]
pub enum CompileErrorKind {
    UnknownMetaVariable(String),
    UnknownMacro(String),
    MissingEntrypoint,
}

#[derive(Debug)]
pub enum CompileError {
    Multi(Vec<CompileError>),
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
