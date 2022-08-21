use std::fs::write;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::{files, util::Fallible};
use std::collections::{BTreeMap, HashSet};

pub fn files_changed<P: AsRef<Path>>(lock_file: P, check_sources: bool) -> Fallible<bool> {
    if lock_file.as_ref().is_file() {
        let lock = Lock::read(&lock_file)?;
        let hashes = if check_sources {
            lock.source_hashes
        } else {
            lock.code_hashes
        };
        let current_hashes = hash_files(hashes.keys())?;
        Ok(current_hashes != hashes)
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

fn hash_files<'a, P: 'a>(files: impl Iterator<Item = &'a P>) -> Fallible<BTreeMap<String, String>>
where
    P: AsRef<Path>,
{
    files
        .map(|p| match hash_file(p) {
            Ok(hash) => match p.as_ref().to_str() {
                None => Err(format!(
                    "Can't hash file due to non-unicode character in path: {}",
                    p.as_ref().display()
                )
                .into()),
                Some(p) => Ok((p.replace('\\', "/"), hash)),
            },
            Err(err) => Err(format!(
                "Unable to hash file: {}\n  Run forced to re-create code files: `yarner --force`.",
                err
            )
            .into()),
        })
        .collect::<Result<BTreeMap<_, _>, _>>()
}

fn hash_file<P: AsRef<Path>>(file: P) -> Fallible<String> {
    let bytes = files::read_file(file.as_ref())?;
    Ok(blake3::hash(&bytes).to_hex().to_string())
}

/// Content for Yarner.lock files
#[derive(Serialize, Deserialize)]
struct Lock {
    source_hashes: BTreeMap<String, String>,
    code_hashes: BTreeMap<String, String>,
}

impl Lock {
    fn read<P: AsRef<Path>>(path: P) -> Fallible<Self> {
        let buf = files::read_file_string(path.as_ref())?;
        let val = toml::from_str::<Self>(&buf).map_err(|err| {
            format!(
                "Invalid lock file {}: {}\n  Delete the file or run with option `--force`.",
                path.as_ref().display(),
                err
            )
        })?;

        Ok(val)
    }

    fn write<P: AsRef<Path>>(&self, path: P) -> Fallible {
        let str = toml::to_string(self)?;
        write(path, str)?;

        Ok(())
    }
}
