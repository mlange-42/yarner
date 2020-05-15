//! Yarner is a general purpose compiler for literate programming, supporting pluggable input and
//! output formats.

#![warn(missing_docs)]

use std::error::Error;
use std::fmt;
use std::path::PathBuf;

pub mod config;
pub mod document;
pub mod parser;
pub mod templates;
mod util;

/// Error type for failed project creation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectCreationError(pub String);

impl Error for ProjectCreationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}
impl fmt::Display for ProjectCreationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Error type for multiple transclusions of the same file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MultipleTransclusionError(pub PathBuf);

impl Error for MultipleTransclusionError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}
impl fmt::Display for MultipleTransclusionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Multiple transclusions of {:?}", self.0)
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
