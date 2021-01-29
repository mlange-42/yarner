//! The built in parsers
//!
//! Additional parsers should implement the `Parser` trait, as well as define a corresponding
//! `ParserConfig` type to allow configuring that parser.
//!
//! Additionally, for each parser, a `Printer` is needed to be able to write the code back
//! out correctly.

pub mod code;
pub mod md;

use std::error::Error;
use std::fmt;

/// A generic parse error
#[derive(Debug)]
pub enum ParseError {
    /// Error for unclosed variables, e.g. @{ without }
    UnclosedVariableError(String),
    /// Error for unclosed transclusions, e.g. @{{ without }}
    UnclosedTransclusionError(String),
    /// Error for invalid transclusions, e.g. if the file is not found
    InvalidTransclusionError(String),
    /// Error for multiple locations with entrypoints to the same code file
    MultipleCodeFileAccessError(String),
} // is there even such a thing as a parse error? who knows.

impl ParseError {
    fn message(&self) -> &str {
        match self {
            ParseError::UnclosedVariableError(s) => s,
            ParseError::UnclosedTransclusionError(s) => s,
            ParseError::InvalidTransclusionError(s) => s,
            ParseError::MultipleCodeFileAccessError(s) => s,
        }
    }
}

impl Error for ParseError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}
impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message())
    }
}
