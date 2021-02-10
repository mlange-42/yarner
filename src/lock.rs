use std::fs::write;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize, Serializer};

use crate::{files, util::Fallible};
use std::collections::{BTreeMap, HashMap, HashSet};

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
    let mut code_hasher = Hasher::default();
    files
        .map(|p| match code_hasher.hash(p) {
            Ok(hash) => Ok((p.clone(), hash)),
            Err(err) => Err(err),
        })
        .collect::<Result<HashMap<_, _>, _>>()
}

/// Content for Yarner.lock files
#[derive(Serialize, Deserialize)]
struct Lock {
    #[serde(serialize_with = "ordered_map")]
    source_hashes: HashMap<PathBuf, String>,
    #[serde(serialize_with = "ordered_map")]
    code_hashes: HashMap<PathBuf, String>,
}

fn ordered_map<S, K, V>(value: &HashMap<K, V>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    K: Ord + Serialize,
    V: Serialize,
{
    let ordered: BTreeMap<_, _> = value.iter().collect();
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
