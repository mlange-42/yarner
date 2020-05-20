//! Config objects, to be read from Yarner.toml

use crate::parser::*;
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;

/// Top-leven config
#[derive(Serialize, Deserialize, Default, Debug)]
pub struct AnyConfig {
    /// Config for Markdown parser
    pub md: Option<MdParser>,
    /// Config for TeX parser
    pub tex: Option<TexParser>,
    /// Config for Html parser
    pub html: Option<HtmlParser>,
    /// Config for paths
    pub paths: Option<Paths>,
    /// Programming language specific settings
    pub language: Option<HashMap<String, LanguageSettings>>,
}

/// Config for paths
#[derive(Serialize, Deserialize, Default, Debug)]
pub struct Paths {
    /// Code output path.
    pub code: Option<String>,
    /// Docs output path.
    pub docs: Option<String>,
    /// The input source file(s) as glob pattern(s).
    pub files: Option<Vec<String>>,
    /// File(s) to include in code output (unprocessed), as glob pattern(s).
    pub code_files: Option<Vec<String>>,
    /// File(s) to include in docs output (unprocessed), as glob pattern(s).
    pub doc_files: Option<Vec<String>>,
    /// Entrypoint block name. Optional. If not supplied, unnamed code blocks are used.
    pub entrypoint: Option<String>,
}

/// Config for a programming language
#[derive(Serialize, Deserialize, Default, Debug)]
pub struct LanguageSettings {
    /// Start of comments in the language
    pub comment_start: String,
    /// Optional end of comments in the language
    pub comment_end: Option<String>,
    /// Start of block labels
    pub block_start: String,
    /// End of block labels
    pub block_end: String,
    /// Determines if code lines containing only whitespace characters are printed as blank lines. Default: true.
    pub clear_blank_lines: bool,
    /// Determines if code files should end with a blank line. Default: true.
    pub eof_newline: bool,
}
