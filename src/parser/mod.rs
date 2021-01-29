//! The built in parsers
//!
//! Additional parsers should implement the `Parser` trait, as well as define a corresponding
//! `ParserConfig` type to allow configuring that parser.
//!
//! Additionally, for each parser, a `Printer` is needed to be able to write the code back
//! out correctly.

pub mod code;
pub mod md;
