use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use toml::Value;

mod document;

pub use document::*;

pub const YARNER_VERSION: &str = env!(
    "CARGO_PKG_VERSION",
    "Environmental variable CARGO_PKG_VERSION not found"
);

#[derive(Debug, Serialize, Deserialize)]
pub struct Context {
    pub config: Value,
    pub name: String,
    pub yarner_version: String,
}

pub fn parse_input() -> serde_json::Result<(Context, HashMap<PathBuf, Document>)> {
    serde_json::from_reader(std::io::stdin())
}

pub fn write_output(documents: &HashMap<PathBuf, Document>) -> serde_json::Result<()> {
    println!("{}", serde_json::to_string_pretty(documents)?);
    Ok(())
}
