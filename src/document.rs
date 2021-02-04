//! The internal representation of a literate document
use crate::config::{LanguageSettings, ParserSettings};
use crate::util::TryCollectExt;
use std::collections::hash_map::HashMap;
use std::fmt::{Display, Formatter, Write};
use std::path::{Path, PathBuf};

/// A representation of a `Document` of literate code
#[derive(Debug)]
pub struct Document {
    pub nodes: Vec<Node>,
}

#[derive(Debug)]
pub enum Node {
    /// A text block
    Text(TextBlock),
    /// A code block
    Code(CodeBlock),
    /// A transclusion
    Transclusion(Transclusion),
}

impl Document {
    /// Creates a new document with the given nodes
    pub fn new(nodes: Vec<Node>) -> Self {
        Document { nodes }
    }

    /// Sets the source file for all code blocks that have none
    pub fn set_source(&mut self, source: &str) {
        for node in &mut self.nodes {
            if let Node::Code(block) = node {
                if block.source_file.is_none() {
                    block.source_file = Some(source.to_owned());
                }
            }
        }
    }

    /// Return the nodes of the document
    pub fn nodes(&self) -> &[Node] {
        &self.nodes
    }

    /// Return the nodes of the document in mutable form
    pub fn nodes_mut(&mut self) -> &mut [Node] {
        &mut self.nodes
    }

    /// Return the nodes of the document, consuming the document
    pub fn into_nodes(self) -> Vec<Node> {
        self.nodes
    }

    /// Gets all the text blocks of this document
    pub fn text_blocks_mut(&mut self) -> impl Iterator<Item = &mut TextBlock> {
        self.nodes.iter_mut().filter_map(|node| match node {
            Node::Text(block) => Some(block),
            _ => None,
        })
    }

    /// Gets all the code blocks of this document
    pub fn code_blocks<'a>(
        &'a self,
        language: Option<&'a str>,
    ) -> impl Iterator<Item = &'a CodeBlock> {
        self.nodes.iter().filter_map(move |node| match node {
            Node::Code(block) => {
                if let (Some(lhs), Some(rhs)) = (language, &block.language) {
                    if lhs != rhs {
                        return None;
                    }
                }

                Some(block)
            }
            _ => None,
        })
    }

    pub fn code_blocks_by_name<'a>(
        &'a self,
        language: Option<&'a str>,
    ) -> HashMap<Option<&'a str>, Vec<&'a CodeBlock>> {
        let mut code_blocks = HashMap::<_, Vec<&CodeBlock>>::new();

        for block in self.code_blocks(language) {
            code_blocks
                .entry(block.name.as_deref())
                .or_default()
                .push(block);
        }

        code_blocks
    }

    /// Gets all the transclusions of this document
    pub fn transclusions(&self) -> impl Iterator<Item = &Transclusion> {
        self.nodes.iter().filter_map(|node| match node {
            Node::Transclusion(trans) => Some(trans),
            _ => None,
        })
    }

    pub fn transclude(
        &mut self,
        replace: &Transclusion,
        with: Document,
        from_source: &str,
        from: &str,
    ) {
        let mut index = 0;
        while index < self.nodes.len() {
            if let Node::Transclusion(trans) = &self.nodes[index] {
                if trans == replace {
                    self.nodes.remove(index);
                    for (i, mut node) in with.nodes.into_iter().enumerate() {
                        if let Node::Code(code) = &mut node {
                            // TODO use entrypoint option here, too? Currently, only in main file.
                            if code.name.is_none() {
                                code.name = Some(from.to_string());
                                code.is_unnamed = true;
                            }
                            // TODO: move to parser?
                            if code.source_file.is_none() {
                                code.source_file = Some(from_source.to_string());
                            }
                        };
                        self.nodes.insert(index + i, node);
                    }
                    // TODO: currently, only a single transclusion of a particular document is possible.
                    // May be sufficient (or even desired), but should be checked.
                    break;
                }
            }
            index += 1;
        }
    }

    /// Finds all file-specific entry points
    pub fn get_entry_points<'a>(
        &'a self,
        settings: &ParserSettings,
        language: Option<&'a str>,
    ) -> HashMap<Option<&'a str>, (&'a Path, Option<PathBuf>)> {
        let mut entries = HashMap::new();
        let pref = &settings.file_prefix;
        for block in self.code_blocks(language) {
            if let Some(name) = block.name.as_deref() {
                if let Some(rest) = name.strip_prefix(pref) {
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
}

/// A `Transclusion` is a reference to another file that should be pulled into the source
#[derive(Debug, PartialEq, Clone)]
pub struct Transclusion {
    /// The target file path
    file: PathBuf,
    /// The original string of the transclusion
    original: String,
}

impl Transclusion {
    /// Creates a new `Transclusion`
    pub fn new(file: PathBuf, original: String) -> Self {
        Transclusion { file, original }
    }

    /// The path to the file this transclusion points to
    pub fn file(&self) -> &PathBuf {
        &self.file
    }

    /// The original string of the transclusion
    pub fn original(&self) -> &str {
        &self.original
    }
}

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

    /// Renders this `TextBlock` as the text it represents
    pub fn lines_mut(&mut self) -> &mut Vec<String> {
        &mut self.text
    }
}

impl Display for TextBlock {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.text.join("\n"))
    }
}

/// A `CodeBlock` is a block of code as defined by the input format.
#[derive(Clone, Default, Debug)]
pub struct CodeBlock {
    /// The indent of this code block is in the documentation file
    pub indent: String,
    /// The name of this code block
    pub name: Option<String>,
    /// Whether the code block was originally unnamed
    pub is_unnamed: bool,
    /// The language this block was written in
    pub language: Option<String>,
    /// Marks the code block as hidden from docs
    pub hidden: bool,
    /// Marks the code block as fenced by alternative sequence
    pub alternative: bool,
    /// The source is the lines of code
    pub source: Vec<Line>,
    /// Source file, for transcluded blocks
    pub source_file: Option<String>,
}

impl CodeBlock {
    /// Creates a new empty `CodeBlock`
    pub fn new() -> Self {
        Self::default()
    }

    /// Indents this code block
    pub fn indented(self, indent: &str) -> Self {
        Self {
            indent: indent.to_owned(),
            ..self
        }
    }

    /// Names this code block
    pub fn named(self, name: String) -> Self {
        Self {
            name: Some(name),
            ..self
        }
    }

    /// Hides this code block
    pub fn hidden(self, hidden: bool) -> Self {
        Self { hidden, ..self }
    }

    /// Marks the code block as fenced by alternative sequence
    pub fn alternative(self, alternative: bool) -> Self {
        Self {
            alternative,
            ..self
        }
    }

    /// Sets the language of this code block
    pub fn in_language(self, language: String) -> Self {
        Self {
            language: Some(language),
            ..self
        }
    }

    /// Adds a line to this code block
    pub fn add_line(&mut self, line: Line) {
        self.source.push(line);
    }

    /// Appends another code block to the end of this one
    pub fn append(&mut self, other: &CodeBlock) {
        self.source.extend_from_slice(&other.source)
    }

    /// "Compiles" this code block into its output code
    pub fn compile(
        &self,
        code_blocks: &HashMap<Option<&str>, Vec<&CodeBlock>>,
        settings: Option<&LanguageSettings>,
    ) -> Result<String, CompileError> {
        self.compile_with(code_blocks, settings)
    }

    /// Returns the line number of the first line in this code block
    pub fn line_number(&self) -> Option<usize> {
        self.source.first().map(|line| line.line_number)
    }

    fn compile_with(
        &self,
        code_blocks: &HashMap<Option<&str>, Vec<&CodeBlock>>,
        settings: Option<&LanguageSettings>,
    ) -> Result<String, CompileError> {
        self.source
            .iter()
            .map(|line| line.compile_with(code_blocks, settings))
            .try_collect()
            .map(|lines| lines.join("\n"))
            .map_err(CompileError::Multi)
    }
}

/// A `Source` represents the source code on a line.
#[derive(Clone, Debug)]
pub enum Source {
    /// A macro invocation, resolved by the literate compiler
    Macro(String),
    /// Source text, possibly including meta variable interpolations
    Source(String),
}

/// A `Line` defines a line of code.
#[derive(Clone, Debug)]
pub struct Line {
    /// The line number of this line (original source)
    pub line_number: usize,
    /// The indent on this line. An indent is defined as leading whitespace (spaces/tabs)
    pub indent: String,
    /// The source text
    pub source: Source,
    /// The literate compiler defined comment - this is extracted from source text to be rendered
    /// in an appropriate format in the documentation, rather than as a comment in the source file
    pub comment: Option<String>,
}

impl Line {
    fn compile_with(
        &self,
        code_blocks: &HashMap<Option<&str>, Vec<&CodeBlock>>,
        settings: Option<&LanguageSettings>,
    ) -> Result<String, CompileError> {
        let comment_start = settings
            .and_then(|s| s.block_labels.as_ref())
            .map(|l| l.comment_start.as_str())
            .unwrap_or_default();
        let comment_end = settings
            .and_then(|s| s.block_labels.as_ref())
            .and_then(|l| l.comment_end.as_deref())
            .unwrap_or_default();
        let block_start = settings
            .and_then(|s| s.block_labels.as_ref())
            .map(|l| l.block_start.as_str())
            .unwrap_or_default();
        let block_end = settings
            .and_then(|s| s.block_labels.as_ref())
            .map(|l| l.block_end.as_str())
            .unwrap_or_default();
        let block_next = settings
            .and_then(|s| s.block_labels.as_ref())
            .map(|l| l.block_next.as_str())
            .unwrap_or("");
        let block_name_sep = '#';

        let clean = if let Some(s) = settings {
            s.clean_code || s.block_labels.is_none()
        } else {
            true
        };

        let blank_lines = settings.map(|s| s.clear_blank_lines).unwrap_or(true);
        match &self.source {
            Source::Source(string) => {
                if blank_lines && string.trim().is_empty() {
                    Ok("".to_string())
                } else {
                    Ok(format!("{}{}", self.indent, string))
                }
            }
            Source::Macro(name) => {
                let blocks = code_blocks.get(&Some(name)).ok_or(CompileError::Single {
                    line_number: self.line_number,
                    kind: CompileErrorKind::UnknownMacro(name.to_string()),
                })?;

                let mut result = String::new();
                for (idx, block) in blocks.iter().enumerate() {
                    let path = block.source_file.to_owned().unwrap_or_default();
                    let name = if block.is_unnamed {
                        ""
                    } else {
                        block.name.as_ref().map(|n| &n[..]).unwrap_or("")
                    };

                    if !clean {
                        writeln!(
                            result,
                            "{}{} {}{}{}{}{}{}{}",
                            &self.indent,
                            comment_start,
                            if idx == 0 { &block_start } else { &block_next },
                            path,
                            block_name_sep,
                            name,
                            block_name_sep,
                            idx,
                            comment_end,
                        )
                        .unwrap();
                    }

                    let code = block.compile_with(code_blocks, settings)?;
                    for line in code.lines() {
                        if blank_lines && line.trim().is_empty() {
                            writeln!(result).unwrap();
                        } else {
                            write!(result, "{}", self.indent).unwrap();
                            writeln!(result, "{}", line).unwrap();
                        }
                    }

                    if !clean && idx == blocks.len() - 1 {
                        writeln!(
                            result,
                            "{}{} {}{}{}{}{}{}{}",
                            &self.indent,
                            comment_start,
                            &block_end,
                            path,
                            block_name_sep,
                            name,
                            block_name_sep,
                            idx,
                            comment_end,
                        )
                        .unwrap();
                    }
                }
                result.pop();
                Ok(result)
            }
        }
    }
}

/// Problems encountered while compiling the document
#[derive(Debug)]
pub enum CompileErrorKind {
    /// An unknown meta variable was encountered
    UnknownMetaVariable(String),
    /// An unknown macro name was encountered
    UnknownMacro(String),
    /// Meta variables incorrect
    InvalidVariables(String),
    /// There is no unnamed code block to use as the entrypoint
    MissingEntrypoint,
}

/// Errors that were encountered while compiling the document
#[derive(Debug)]
pub enum CompileError {
    #[doc(hidden)]
    Multi(Vec<CompileError>),
    #[doc(hidden)]
    Single {
        line_number: usize,
        kind: CompileErrorKind,
    },
}

impl std::fmt::Display for CompileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompileError::Multi(errors) => {
                for error in errors {
                    writeln!(f, "{}", error)?;
                }
                Ok(())
            }
            CompileError::Single { line_number, kind } => {
                writeln!(f, "{:?} (line {})", kind, line_number)
            }
        }
    }
}

impl std::error::Error for CompileError {}
