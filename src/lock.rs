use std::fs::write;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize, Serializer};

use crate::{files, util::Fallible};
use std::collections::{BTreeMap, HashMap, HashSet};

pub fn sources_changed<P: AsRef<Path>>(lock_file: P) -> Fallible<bool> {
    if lock_file.as_ref().is_file() {
        let lock = Lock::read(lock_file)?;
        let source_hashes = hash_files(lock.source_hashes.keys())?;
        Ok(source_hashes != lock.source_hashes)
    } else {
        Ok(false)
    }
}

pub fn code_changed<P: AsRef<Path>>(lock_file: P) -> Fallible<bool> {
    if lock_file.as_ref().is_file() {
        let lock = Lock::read(lock_file)?;
        let code_hashes = hash_files(lock.code_hashes.keys())?;
        Ok(code_hashes != lock.code_hashes)
    } else {
        Ok(false)
    }
}

pub fn write_lock<P: AsRef<Path>>(
    lock_file: P,
    source_files: &HashSet<PathBuf>,
    code_files: &HashSet<PathBuf>,
) -> Fallible {
    let lock = Lock {
        source_hashes: hash_files(source_files.iter())?,
        code_hashes: hash_files(code_files.iter())?,
    };
    lock.write(&lock_file)
}

fn hash_files<'a>(files: impl Iterator<Item = &'a PathBuf>) -> Fallible<HashMap<PathBuf, String>> {
    files
        .map(|p| match hash_file(p) {
            Ok(hash) => Ok((p.clone(), hash)),
            Err(err) => Err(err),
        })
        .collect::<Result<HashMap<_, _>, _>>()
}

fn hash_file<P: AsRef<Path>>(file: P) -> Fallible<String> {
    let mut hasher = blake3::Hasher::new();
    let bytes = files::read_file(file.as_ref())?;
    hasher.update(&bytes);
    Ok(hasher.finalize().to_hex().to_string())
}

/// Content for Yarner.lock files
#[derive(Serialize, Deserialize)]
struct Lock {
    #[serde(serialize_with = "ordered_map")]
    source_hashes: HashMap<PathBuf, String>,
    #[serde(serialize_with = "ordered_map")]
    code_hashes: HashMap<PathBuf, String>,
}

fn ordered_map<S>(value: &HashMap<PathBuf, String>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let ordered: BTreeMap<_, _> = value
        .iter()
        .map(|(k, v)| (k.to_str().unwrap_or("non-uft8-path").replace('\\', "/"), v))
        .collect();
    ordered.serialize(serializer)
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
