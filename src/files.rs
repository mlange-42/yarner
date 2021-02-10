use std::{
    collections::{
        hash_map::Entry::{Occupied, Vacant},
        HashMap,
    },
    ffi::OsStr,
    iter::repeat,
    path::{Path, PathBuf},
};

use crate::util::Fallible;
use std::collections::HashSet;

pub fn read_file_string(path: &Path) -> Fallible<String> {
    std::fs::read_to_string(&path).map_err(|err| format!("{}: {}", err, path.display()).into())
}

pub fn read_file(path: &Path) -> Fallible<Vec<u8>> {
    std::fs::read(&path).map_err(|err| format!("{}: {}", err, path.display()).into())
}

pub fn copy_files(
    patterns: &[String],
    path_mod: Option<&[String]>,
    target_dir: &Path,
    reverse: bool,
) -> Result<(HashSet<PathBuf>, HashSet<PathBuf>), String> {
    match path_mod {
        Some(path_mod) if patterns.len() != path_mod.len() => {
            return Err(
                "If argument code_paths/doc_paths is given in the toml file, it must have as many elements as argument code_files/doc_files".to_string()
            );
        }
        _ => (),
    }
    let mut track_copy_dest: HashMap<PathBuf, PathBuf> = HashMap::new();
    let mut source_files: HashSet<PathBuf> = HashSet::new();
    let mut out_files: HashSet<PathBuf> = HashSet::new();
    for (idx, file_pattern) in patterns.iter().enumerate() {
        let path = path_mod.as_ref().map(|paths| &paths[idx]);
        let paths = match glob::glob(&file_pattern) {
            Ok(p) => p,
            Err(err) => {
                return Err(format!(
                    "Unable to parse glob pattern \"{}\" (at index {}): {}",
                    file_pattern, err.pos, err
                ))
            }
        };
        for p in paths {
            let file = match p {
                Ok(p) => p,
                Err(err) => {
                    return Err(format!(
                        "Unable to access result found by glob pattern \"{}\" (at {}): {}",
                        file_pattern,
                        err.path().display(),
                        err
                    ))
                }
            };
            if file.is_file() {
                let out_path = path.map_or(file.clone(), |path| modify_path(&file, &path));
                match track_copy_dest.entry(out_path.clone()) {
                    Occupied(entry) => {
                        return Err(format!(
                            "Attempted to copy multiple code files to {}: from {} and {}",
                            out_path.display(),
                            entry.get().display(),
                            file.display()
                        ));
                    }
                    Vacant(entry) => {
                        entry.insert(file.clone());
                    }
                }

                let mut file_path = target_dir.to_owned();
                file_path.push(out_path);

                if !reverse {
                    std::fs::create_dir_all(file_path.parent().unwrap()).unwrap();
                }
                let (from, to) = if reverse {
                    println!("Copying file {} to {}", file_path.display(), file.display());
                    (&file_path, &file)
                } else {
                    println!("Copying file {} to {}", file.display(), file_path.display());
                    (&file, &file_path)
                };
                if let Err(err) = std::fs::copy(&from, &to) {
                    return Err(format!("Error copying file {}: {}", file.display(), err));
                }
                source_files.insert(from.to_owned());
                out_files.insert(to.to_owned());
            }
        }
    }
    Ok((source_files, out_files))
}

fn modify_path(path: &Path, replace: &str) -> PathBuf {
    if replace.is_empty() || replace == "_" {
        return path.to_owned();
    }

    let replace = Path::new(replace)
        .components()
        .map(|comp| comp.as_os_str())
        .chain(repeat(OsStr::new("_")));

    let mut modified = PathBuf::new();

    for (comp, replace) in path.components().zip(replace) {
        if replace == "_" {
            modified.push(comp);
        } else if replace != "-" {
            modified.push(replace);
        }
    }

    modified
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unmodified_path() {
        assert_eq!(
            modify_path(Path::new("foo/bar/baz.qux"), ""),
            Path::new("foo/bar/baz.qux")
        );

        assert_eq!(
            modify_path(Path::new("foo/bar/baz.qux"), "_"),
            Path::new("foo/bar/baz.qux")
        );

        assert_eq!(
            modify_path(Path::new("foo/bar/baz.qux"), "_/_/_"),
            Path::new("foo/bar/baz.qux")
        );

        assert_eq!(
            modify_path(Path::new("foo/bar/baz.qux"), "_/_"),
            Path::new("foo/bar/baz.qux")
        );

        assert_eq!(
            modify_path(Path::new("foo/bar/baz.qux"), "_/_/_/_"),
            Path::new("foo/bar/baz.qux")
        );
    }

    #[test]
    fn drop_component_from_path() {
        assert_eq!(
            modify_path(Path::new("foo/bar/baz.qux"), "-/_/_"),
            Path::new("bar/baz.qux")
        );

        assert_eq!(
            modify_path(Path::new("foo/bar/baz.qux"), "_/-/_"),
            Path::new("foo/baz.qux")
        );

        assert_eq!(
            modify_path(Path::new("foo/bar/baz.qux"), "_/_/-"),
            Path::new("foo/bar")
        );

        assert_eq!(
            modify_path(Path::new("foo/bar/baz.qux"), "_/_/_/-"),
            Path::new("foo/bar/baz.qux")
        );
    }

    #[test]
    fn replace_component_in_path() {
        assert_eq!(
            modify_path(Path::new("foo/bar/baz.qux"), "FOO/_/_"),
            Path::new("FOO/bar/baz.qux")
        );

        assert_eq!(
            modify_path(Path::new("foo/bar/baz.qux"), "_/BAR/_"),
            Path::new("foo/BAR/baz.qux")
        );

        assert_eq!(
            modify_path(Path::new("foo/bar/baz.qux"), "_/_/BAZ"),
            Path::new("foo/bar/BAZ")
        );

        assert_eq!(
            modify_path(Path::new("foo/bar/baz.qux"), "_/_/_/QUX"),
            Path::new("foo/bar/baz.qux")
        );
    }
}
