//! Representation of the text parts of the AST

/// A `TextBlock` is just text that will be copied verbatim into the output documentation file
#[derive(Debug, Default)]
pub struct TextBlock<'a> {
    /// The source text
    text: Vec<&'a str>,
}

impl<'a> TextBlock<'a> {
    /// Creates a new empty `TextBlock`
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a line to this `TextBlock`
    pub fn add_line(&mut self, line: &'a str) {
        self.text.push(line);
    }

    /// Renders this `TextBlock` as the text it represents
    pub fn to_string(&self) -> String {
        self.text.join("\n")
    }

    /// Renders this `TextBlock` as the text it represents
    pub fn lines(&self) -> &Vec<&'a str> {
        &self.text
    }
}
