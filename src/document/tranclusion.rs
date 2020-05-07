//! Representation of transclusions of the AST, to pull in other files.

use std::path::PathBuf;

/// A `Transclusion` is a reference to another file that should be pulled into the source
#[derive(Debug)]
pub struct Transclusion<'a> {
    /// The target file path
    file: &'a PathBuf,
}

impl<'a> Transclusion<'a> {
    /// Creates a new `Transclusion`
    pub fn new(file: &'a PathBuf) -> Self {
        Transclusion { file }
    }
}
