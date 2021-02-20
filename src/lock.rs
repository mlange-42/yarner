use std::collections::{BTreeMap, HashSet};
use std::path::{Path, PathBuf};

use yarner_lib::config::Lock;

use crate::{files, util::Fallible};

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
                err.to_string()
            )
            .into()),
        })
        .collect::<Result<BTreeMap<_, _>, _>>()
}

fn hash_file<P: AsRef<Path>>(file: P) -> Fallible<String> {
    let bytes = files::read_file(file.as_ref())?;
    Ok(blake3::hash(&bytes).to_hex().to_string())
}
