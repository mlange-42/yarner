use std::fs::{self, read_to_string, write, File};
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};

use md5::Context;
use serde::{Deserialize, Serialize};

use crate::util::Fallible;

pub fn code_changed<P: AsRef<Path>>(lock_file: P, code_dir: P) -> std::io::Result<bool> {
    if code_dir.as_ref().exists()
        && code_dir.as_ref().is_dir()
        && lock_file.as_ref().exists()
        && lock_file.as_ref().is_file()
    {
        let mut code_hasher = Hasher::default();
        code_hasher.consume_all(code_dir)?;
        let code_hash = code_hasher.compute();

        let lock = Lock::read(lock_file)?;

        Ok(lock.code_hash != code_hash)
    } else {
        Ok(false)
    }
}

/// Content for Yarner.lock files
#[derive(Serialize, Deserialize)]
pub struct Lock {
    pub source_hash: String,
    pub code_hash: String,
}

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

    pub fn compute(self) -> String {
        hex::encode(self.hasher.compute().as_ref())
    }

    pub fn consume_file<P: AsRef<Path>>(&mut self, file: P) -> std::io::Result<()> {
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
