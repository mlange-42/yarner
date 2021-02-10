use std::fs::{self, write};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::{files, util::Fallible};

pub fn code_changed<P: AsRef<Path>>(lock_file: P, code_dir: P) -> Fallible<bool> {
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

pub fn write_lock<P: AsRef<Path>>(lock_file: P, code_dir: P) -> Fallible {
    let mut code_hasher = Hasher::default();
    code_hasher.consume_all(code_dir)?;
    let code_hash = code_hasher.compute();

    let lock = Lock {
        code_hash,
        source_hash: String::new(),
    };
    lock.write(&lock_file)
}

/// Content for Yarner.lock files
#[derive(Serialize, Deserialize)]
struct Lock {
    source_hash: String,
    code_hash: String,
}

impl Lock {
    fn read<P: AsRef<Path>>(path: P) -> Fallible<Self> {
        let buf = files::read_file_string(path.as_ref())?;
        let val = toml::from_str::<Self>(&buf)?;

        Ok(val)
    }

    fn write<P: AsRef<Path>>(&self, path: P) -> Fallible {
        let str = toml::to_string(self)?;
        write(path, str)?;

        Ok(())
    }
}

struct Hasher {
    hasher: blake3::Hasher,
}

impl Default for Hasher {
    fn default() -> Self {
        Self {
            hasher: blake3::Hasher::new(),
        }
    }
}

impl Hasher {
    fn consume_all<P: AsRef<Path>>(&mut self, path: P) -> Fallible {
        let root = PathBuf::from(path.as_ref());
        if root.is_dir() {
            let mut entries = fs::read_dir(root)?.collect::<Result<Vec<_>, _>>()?;
            entries.sort_by_cached_key(|d| d.path());
            for entry in entries {
                self.consume_all(entry.path())?;
            }
        } else {
            self.consume_file(root)?;
        }
        Ok(())
    }

    fn compute(self) -> String {
        self.hasher.finalize().to_hex().to_string()
    }

    fn consume_file<P: AsRef<Path>>(&mut self, file: P) -> Fallible {
        let bytes = files::read_file(file.as_ref())?;
        self.hasher.update(&bytes);

        Ok(())
    }
}
