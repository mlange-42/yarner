use crate::document::Document;
use crate::util::Fallible;
use md5::Context;
use serde::{Deserialize, Serialize};
use std::fs::{self, read_to_string, write, File};
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};

/// Content for Yarner.lock files
#[derive(Serialize, Deserialize)]
pub struct Lock {
    pub source_hash: String,
    pub code_hash: String,
}

// TODO: remove when in use
#[allow(dead_code)]
impl Lock {
    pub fn read<P: AsRef<Path>>(path: P) -> std::io::Result<Self> {
        let buf = read_to_string(path)?;
        let val = toml::from_str::<Self>(&buf)?;

        Ok(val)
    }

    pub fn write<P: AsRef<Path>>(&self, path: P) -> Fallible {
        let str = toml::to_string(self)?;
        write(path, str)?;

        Ok(())
    }

    pub fn has_changed(&self, hasher: Hasher, code: bool) -> std::io::Result<bool> {
        let old_hash = if code {
            &self.code_hash
        } else {
            &self.source_hash
        };
        Ok(old_hash == &hasher.compute())
    }
}

pub struct Hasher {
    hasher: Context,
}

impl Default for Hasher {
    fn default() -> Self {
        Self {
            hasher: Context::new(),
        }
    }
}

// TODO: remove when in use
#[allow(dead_code)]
impl Hasher {
    pub fn consume_all<P: AsRef<Path>>(&mut self, path: P) -> std::io::Result<()> {
        let root = PathBuf::from(path.as_ref());
        if root.is_dir() {
            for entry in fs::read_dir(root)? {
                let entry = entry?;
                self.consume_all(entry.path())?;
            }
        } else {
            self.consume_file(root)?;
        }
        Ok(())
    }

    pub fn consume_doc(&mut self, document: &Document) -> Fallible {
        for block in document.code_blocks(None) {
            let bytes = bincode::serialize(block)?;
            self.hasher.consume(&bytes);
        }
        Ok(())
    }

    pub fn compute(self) -> String {
        hex::encode(self.hasher.compute().as_ref())
    }

    fn consume_file<P: AsRef<Path>>(&mut self, file: P) -> std::io::Result<()> {
        let f = File::open(file)?;
        let mut buffer = [0; 1024];
        let mut reader = BufReader::new(f);

        loop {
            let count = reader.read(&mut buffer)?;
            if count == 0 {
                break;
            }
            self.hasher.consume(&buffer[..count]);
        }

        Ok(())
    }
}
