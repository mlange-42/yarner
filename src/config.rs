//! Config objects, to be read from Yarner.toml

use crate::parser::md::MdParser;
use crate::util::Fallible;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::read_to_string;
use std::path::Path;
use toml::from_str;

/// Top-level config
#[derive(Deserialize, Debug)]
pub struct Config {
    /// Config for Markdown parser
    pub parser: MdParser,
    /// Config for paths
    #[serde(default)]
    pub paths: Paths,
    /// Programming language specific settings
    #[serde(default)]
    pub language: HashMap<String, LanguageSettings>,
}

impl Config {
    pub fn read<P: AsRef<Path>>(path: P) -> Fallible<Self> {
        let buf = read_to_string(path)?;
        let val = from_str::<Self>(&buf)?;

        val.check()?;

        Ok(val)
    }

    /// Check the validity of the configuration
    fn check(&self) -> Fallible {
        for language in self.language.values() {
            language.check()?;
        }

        Ok(())
    }
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
    /// Separator between block source and name
    pub name_separator: String,
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
