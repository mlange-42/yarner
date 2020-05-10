//! Config objects, to be read from Outline.toml

use crate::parser::*;
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;

/// Top-leven config
#[derive(Serialize, Deserialize, Default, Debug)]
pub struct AnyConfig {
    /// Config for Bird parser
    pub bird: Option<BirdParser>,
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
    /// Code output path
    pub code: Option<String>,
    /// Docs output path
    pub docs: Option<String>,
    /// Input files
    pub files: Option<Vec<String>>,
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
    /// Determines if code lines containing only whitespace characters are printed as blank lines.
    pub blank_lines: bool,
}
