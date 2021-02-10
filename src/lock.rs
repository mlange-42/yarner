use std::fs::write;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::{files, util::Fallible};
use std::collections::{HashMap, HashSet};

pub fn code_changed<P: AsRef<Path>>(lock_file: P) -> Fallible<bool> {
    if lock_file.as_ref().exists() && lock_file.as_ref().is_file() {
        let _lock = Lock::read(lock_file)?;
        Ok(false)
    } else {
        Ok(false)
    }
}

pub fn write_lock<P: AsRef<Path>>(lock_file: P, code_files: HashSet<PathBuf>) -> Fallible {
    let mut code_hasher = Hasher::default();

    let code_hashes = code_files
        .iter()
        .map(|p| match code_hasher.hash(p) {
            Ok(hash) => Ok((p.clone(), hash)),
            Err(err) => Err(err),
        })
        .collect::<Result<HashMap<_, _>, _>>()?;

    let lock = Lock {
        code_hashes,
        source_hashes: HashMap::new(),
    };
    lock.write(&lock_file)
}

/// Content for Yarner.lock files
#[derive(Serialize, Deserialize)]
struct Lock {
    source_hashes: HashMap<PathBuf, String>,
    code_hashes: HashMap<PathBuf, String>,
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
    fn hash<P: AsRef<Path>>(&mut self, file: P) -> Fallible<String> {
        self.hasher.reset();
        self.consume_file(file)?;
        let result = self.hasher.finalize().to_hex().to_string();
        self.hasher.reset();
        Ok(result)
    }

    fn consume_file<P: AsRef<Path>>(&mut self, file: P) -> Fallible {
        let bytes = files::read_file(file.as_ref())?;
        self.hasher.update(&bytes);

        Ok(())
    }
}
