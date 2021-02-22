//! Config objects, to be read from Yarner.toml
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use once_cell::sync::Lazy;
use regex::Regex;
use serde::{de::Error as _, Deserialize, Deserializer};

use crate::{files, util::Fallible};
use toml::value::Table;

pub const LINK_PATTERN: &str = r"\[([^\[\]]*)\]\((.*?)\)";
pub static LINK_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(LINK_PATTERN).unwrap());

pub const CRLF_NEWLINE: &str = "\r\n";
pub const LF_NEWLINE: &str = "\n";

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
    /// TOML table of settings for pre-processors
    #[serde(default)]
    pub preprocessor: Table,
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

    pub fn has_reverse_config(&self) -> bool {
        self.language
            .values()
            .any(|lang| lang.block_labels.is_some())
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
}

fn from_link_prefix<'de, D>(deserializer: D) -> Result<(String, Regex), D::Error>
where
    D: Deserializer<'de>,
{
    let prefix: &str = Deserialize::deserialize(deserializer)?;
    Ok((
        prefix.to_string(),
        Regex::new(&format!("({})?{}", prefix, LINK_PATTERN)).map_err(|err| {
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
#[derive(Deserialize, Default, Debug, Clone)]
pub struct Paths {
    /// Code output path.
    pub root: Option<String>,
    /// Code output path.
    pub code: Option<PathBuf>,
    /// Docs output path.
    pub docs: Option<PathBuf>,
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

impl Paths {
    pub fn has_valid_code_path(&self) -> bool {
        self.code.as_ref().map(|d| d.is_dir()).unwrap_or(false)
    }
}

/// Config for a programming language
#[derive(Deserialize, Default, Debug)]
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
#[derive(Deserialize, Default, Debug)]
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

    pub fn label_prefixes(&self) -> (String, String, String) {
        let start = format!("{} {}", self.comment_start, self.block_start);
        let next = format!("{} {}", self.comment_start, self.block_next);
        let end = format!("{} {}", self.comment_start, self.block_end);

        (start, next, end)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CONFIG: &str = include_str!("create/Yarner.toml");

    #[test]
    fn config_template() {
        let config = toml::from_str::<Config>(CONFIG).unwrap();
        config.check().unwrap();
    }

    #[test]
    fn label_prefixes() {
        let labels = default_block_labels();
        let (start, next, end) = labels.label_prefixes();
        assert_eq!(start, "// <@");
        assert_eq!(next, "// <@>");
        assert_eq!(end, "// @>");
    }

    #[test]
    #[should_panic]
    fn block_labels_check() {
        let labels = BlockLabels {
            comment_start: "//".to_string(),
            comment_end: None,
            block_start: "<@|".to_string(),
            block_next: "<@".to_string(),
            block_end: "@>".to_string(),
        };
        labels.check().unwrap();
    }

    fn default_block_labels() -> BlockLabels {
        BlockLabels {
            comment_start: "//".to_string(),
            comment_end: None,
            block_start: "<@".to_string(),
            block_next: "<@>".to_string(),
            block_end: "@>".to_string(),
        }
    }
}
