//! Representation of transclusions of the AST, to pull in other files.

use std::path::PathBuf;

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
