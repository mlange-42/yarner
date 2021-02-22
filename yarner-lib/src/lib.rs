use std::collections::HashMap;
use std::path::PathBuf;
use toml::Value;

mod document;

pub use document::*;

pub fn parse_input() -> serde_json::Result<(Value, HashMap<PathBuf, Document>)> {
    serde_json::from_reader(std::io::stdin())
}

pub fn write_output(documents: &HashMap<PathBuf, Document>) -> serde_json::Result<()> {
    println!("{}", serde_json::to_string_pretty(documents)?);
    Ok(())
}
