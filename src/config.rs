//! Config objects, to be read from Yarner.toml
use std::collections::HashMap;
use std::path::Path;

use once_cell::sync::Lazy;
use regex::Regex;
use serde::{de::Error as _, Deserialize, Deserializer, Serialize};

use crate::{files, util::Fallible};

pub(crate) const LINK_PATTERN: &str = r"\[([^\[\]]*)\]\((.*?)\)";
pub(crate) static LINK_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(LINK_PATTERN).unwrap());

/// Top-level config
#[derive(Deserialize, Debug)]
pub struct Config {
    /// Config for Markdown parser
    pub parser: ParserSettings,
    /// Config for paths
    #[serde(default)]
    pub paths: Paths,
    /// Programming language specific settings
    #[serde(default)]
    pub language: HashMap<String, LanguageSettings>,
}

impl Config {
    pub fn read<P: AsRef<Path>>(path: P) -> Fallible<Self> {
        let buf = files::read_file_string(path.as_ref())?;
        let val = toml::from_str::<Self>(&buf)?;

        Ok(val)
    }

    /// Check the validity of the configuration
    pub fn check(&self) -> Fallible {
        self.parser.check()?;
        for language in self.language.values() {
            language.check()?;
        }

        Ok(())
    }
}

/// The config for parsing a Markdown document
#[derive(Clone, Deserialize, Debug)]
pub struct ParserSettings {
    /// The sequence that identifies the start and end of a fenced code block
    pub fence_sequence: String,
    /// Alternative sequence that identifies the start and end of a fenced code block.
    /// Allows for normal Markdown fences in code blocks
    pub fence_sequence_alt: String,
    /// The language to set if there was no automatically detected language. Optional
    pub default_language: Option<String>,
    /// Temporary switch to disable comment extraction
    #[serde(default)]
    pub comments_as_aside: bool,
    /// The sequence to identify a comment which should be omitted from the compiled code, and may
    /// be rendered as an `<aside>` if `comments_as_aside` is set.
    pub block_name_prefix: String,
    /// The sequence to identify the start of a macro invocation.
    pub macro_start: String,
    /// The sequence to identify the end of a macro invocation.
    pub macro_end: String,
    /// The sequence to identify the start of a transclusion.
    pub transclusion_start: String,
    /// The sequence to identify the end of a transclusion.
    pub transclusion_end: String,
    /// Prefix for links that should be followed during processing.
    /// Should be RegEx-compatible.
    #[serde(rename(deserialize = "link_prefix"))]
    #[serde(deserialize_with = "from_link_prefix")]
    pub link_following_pattern: (String, Regex),
    /// Prefix for file-specific entry points.
    pub file_prefix: String,
    /// Name prefix for code blocks not shown in the docs.
    pub hidden_prefix: String,
}

impl ParserSettings {
    pub fn check(&self) -> Result<(), String> {
        if self.comments_as_aside {
            Err(r#"Comment extraction is temporarily disabled.
Please comment out option `comments_as_aside` until the next version, and rename `comment_start` to `block_name_prefix`"#.to_string())
        } else {
            Ok(())
        }
    }

    /// Sets the default language of the returned parser (or does nothing if `None` is passed)
    pub fn default_language(&self, language: Option<String>) -> Self {
        let mut cloned = self.clone();

        if language.is_some() {
            cloned.default_language = language;
        }

        cloned
    }
}

fn from_link_prefix<'de, D>(deserializer: D) -> Result<(String, Regex), D::Error>
where
    D: Deserializer<'de>,
{
    let prefix: &str = Deserialize::deserialize(deserializer)?;
    Ok((
        prefix.to_string(),
        Regex::new(&format!("{}{}", prefix, LINK_PATTERN)).map_err(|err| {
            D::Error::custom(format!(
                "Error compiling Regex pattern {}{}\n{}",
                prefix,
                LINK_PATTERN,
                err.to_string()
            ))
        })?,
    ))
}

/// Config for paths
#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct Paths {
    /// Code output path.
    pub root: Option<String>,
    /// Code output path.
    pub code: Option<String>,
    /// Docs output path.
    pub docs: Option<String>,
    /// The input source file(s) as glob pattern(s).
    pub files: Option<Vec<String>>,
    /// File(s) to include in code output (unprocessed), as glob pattern(s).
    pub code_files: Option<Vec<String>>,
    /// Replacement of path components to modify code paths.
    pub code_paths: Option<Vec<String>>,
    /// File(s) to include in docs output (unprocessed), as glob pattern(s).
    pub doc_files: Option<Vec<String>>,
    /// Replacement of path components to modify doc paths.
    pub doc_paths: Option<Vec<String>>,
    /// Entrypoint block name. Optional. If not supplied, unnamed code blocks are used.
    pub entrypoint: Option<String>,
}

/// Config for a programming language
#[derive(Serialize, Deserialize, Default, Debug)]
pub struct LanguageSettings {
    /// Label format for blocks in code output
    pub block_labels: Option<BlockLabels>,
    /// Determines if code lines containing only whitespace characters are printed as blank lines. Default: true.
    pub clear_blank_lines: bool,
    /// Determines if code files should end with a blank line. Default: true.
    pub eof_newline: bool,
    /// Print code without block labels.
    #[serde(skip)]
    pub clean_code: bool,
}

impl LanguageSettings {
    /// Check the validity of language settings
    fn check(&self) -> Fallible {
        if let Some(labels) = &self.block_labels {
            labels.check()
        } else {
            Ok(())
        }
    }
}

/// Config for block labels for a programming language
#[derive(Serialize, Deserialize, Default, Debug)]
pub struct BlockLabels {
    /// Start of comments in the language
    pub comment_start: String,
    /// Optional end of comments in the language
    pub comment_end: Option<String>,
    /// Start of block labels
    pub block_start: String,
    /// Start of next block of the same name
    pub block_next: String,
    /// End of block labels
    pub block_end: String,
}

impl BlockLabels {
    /// Check the validity of block label settings
    fn check(&self) -> Fallible {
        if self.block_start.starts_with(&self.block_next) {
            return Err(
                "Language parameter 'block_start' must not start with the same sequence as 'block_next'"
                    .into(),
            );
        }
        if self.block_end.starts_with(&self.block_start) {
            return Err(
                "Language parameter 'block_end' must not start with the same sequence as 'block_start'"
                    .into(),
            );
        }
        Ok(())
    }
}
