//! Representation of the text parts of the AST

use core::fmt::Formatter;
use std::fmt::Display;

/// A `TextBlock` is just text that will be copied verbatim into the output documentation file
#[derive(Debug, Default)]
pub struct TextBlock {
    /// The source text
    text: Vec<String>,
}

impl TextBlock {
    /// Creates a new empty `TextBlock`
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a line to this `TextBlock`
    pub fn add_line(&mut self, line: &str) {
        self.text.push(line.to_owned());
    }

    /// Renders this `TextBlock` as the text it represents
    pub fn lines(&self) -> &Vec<String> {
        &self.text
    }
}

impl Display for TextBlock {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.text.join("\n"))
    }
}
