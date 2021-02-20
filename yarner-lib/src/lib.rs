use crate::document::Document;
use std::collections::HashMap;
use std::io::Read;
use std::path::PathBuf;
use toml::Value;

pub mod config;
pub mod document;

pub fn to_json(
    config: &Value,
    documents: &HashMap<PathBuf, Document>,
) -> serde_json::Result<String> {
    serde_json::to_string_pretty(&(config, documents))
}

pub fn from_json(json: &str) -> serde_json::Result<(Value, HashMap<PathBuf, Document>)> {
    serde_json::from_str(json)
}

pub fn parse_input<R: Read>(reader: R) -> serde_json::Result<(Value, HashMap<PathBuf, Document>)> {
    serde_json::from_reader(reader)
}
