//! Config objects, to be read from Outline.toml

use crate::parser::*;
use serde_derive::Deserialize;

/// Top-leven config
#[derive(Deserialize, Default, Debug)]
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
}

/// Config for paths
#[derive(Deserialize, Default, Debug)]
pub struct Paths {
    /// Code output path
    pub code: Option<String>,
    /// Docs output path
    pub docs: Option<String>,
    /// Input files
    pub files: Option<Vec<String>>,
}
