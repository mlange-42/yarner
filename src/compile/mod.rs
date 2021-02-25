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
