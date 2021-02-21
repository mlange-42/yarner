use std::collections::HashMap;
use std::path::{Path, PathBuf};
use yarner_lib::{Document, Node};

pub mod forward;
pub mod reverse;

/// Sets the source file for all code blocks that have none
fn set_source(document: &mut Document, source: &str) {
    for node in &mut document.nodes {
        if let Node::Code(block) = node {
            if block.source_file.is_none() {
                block.source_file = Some(source.to_owned());
            }
        }
    }
}

/// Finds all file-specific entry points
fn entry_points<'a>(
    document: &'a Document,
    file_prefix: &str,
) -> HashMap<Option<&'a str>, (&'a Path, Option<PathBuf>)> {
    let mut entries = HashMap::new();
    for block in document.code_blocks() {
        if let Some(name) = block.name.as_deref() {
            if let Some(rest) = name.strip_prefix(file_prefix) {
                entries.insert(
                    Some(name),
                    (
                        Path::new(rest),
                        block.source_file.as_ref().map(|file| file.into()),
                    ),
                );
            }
        }
    }
    entries
}
