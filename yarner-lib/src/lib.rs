use crate::config::Config;
use crate::document::Document;
use std::collections::HashMap;
use std::path::PathBuf;

pub mod config;
pub mod document;

#[allow(dead_code)]
fn to_json(config: &Config, documents: &HashMap<PathBuf, Document>) -> serde_json::Result<String> {
    serde_json::to_string_pretty(&(config, documents))
}
