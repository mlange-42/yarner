use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use toml::Value;

mod document;

pub use document::*;

/// Version of this library
pub const YARNER_VERSION: &str = env!(
    "CARGO_PKG_VERSION",
    "Environmental variable CARGO_PKG_VERSION not found"
);

/// Pre-processor call context
#[derive(Debug, Serialize, Deserialize)]
pub struct Context {
    /// Configuration of the pre-processor
    pub config: Value,
    /// Name of the pre-processor
    pub name: String,
    /// Yarner version from from which the pre-processor is called
    pub yarner_version: String,
}

/// Read inputs from STDIN and parse into Context and Documents
pub fn parse_input() -> serde_json::Result<(Context, HashMap<PathBuf, Document>)> {
    serde_json::from_reader(std::io::stdin())
}

/// Write Documents as JSON to STDOUT
pub fn write_output(documents: &HashMap<PathBuf, Document>) -> serde_json::Result<()> {
    println!("{}", serde_json::to_string_pretty(documents)?);
    Ok(())
}
