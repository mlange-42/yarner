use log::info;
use std::{
    collections::{
        hash_map::Entry::{Occupied, Vacant},
        HashMap, HashSet,
    },
    ffi::OsStr,
    iter::repeat,
    path::{Path, PathBuf},
};

use crate::util::Fallible;

pub fn read_file_string(path: &Path) -> Fallible<String> {
    std::fs::read_to_string(&path).map_err(|err| format!("{}: {}", err, path.display()).into())
}

pub fn read_file(path: &Path) -> Fallible<Vec<u8>> {
    std::fs::read(&path).map_err(|err| format!("{}: {}", err, path.display()).into())
}

fn files_differ(old: &Path, new: &Path) -> bool {
    read_file(old)
        .and_then(|old| read_file(new).map(|new| old != new))
        .unwrap_or(true)
}

pub fn file_differs(file: &Path, new_content: &str) -> bool {
    read_file_string(file).map_or(true, |content| content != new_content)
}

pub fn copy_files(
    patterns: &[String],
    path_mod: Option<&[String]>,
    target_dir: &Path,
    reverse: bool,
) -> Fallible<(HashSet<PathBuf>, HashSet<PathBuf>)> {
    match path_mod {
        Some(path_mod) if patterns.len() != path_mod.len() => {
            return Err(
                "If argument code_paths/doc_paths is given in the toml file, it must have as many elements as argument code_files/doc_files".into()
            );
        }
        _ => (),
    }
    let mut track_copy_dest: HashMap<PathBuf, PathBuf> = HashMap::new();
    for (idx, file_pattern) in patterns.iter().enumerate() {
        let path = path_mod.as_ref().map(|paths| &paths[idx]);
        let paths = glob::glob(file_pattern).map_err(|err| {
            format!(
                "Unable to parse glob pattern \"{}\" (at index {}): {}",
                file_pattern, err.pos, err
            )
        })?;

        for p in paths {
            let file = p.map_err(|err| {
                format!(
                    "Unable to access result found by glob pattern \"{}\" (at {}): {}",
                    file_pattern,
                    err.path().display(),
                    err
                )
            })?;

            if file.is_file() {
                let out_path = path.map_or(file.clone(), |path| modify_path(&file, path));
                let mut file_path = target_dir.to_owned();
                file_path.push(out_path);

                match track_copy_dest.entry(file_path.clone()) {
                    Occupied(entry) => {
                        return Err(format!(
                            "Attempted to copy multiple code files to {}: from {} and {}",
                            file_path.display(),
                            entry.get().display(),
                            file.display()
                        )
                        .into());
                    }
                    Vacant(entry) => {
                        entry.insert(file.clone());
                    }
                }

                if !reverse {
                    std::fs::create_dir_all(file_path.parent().unwrap()).unwrap();
                }
                let (from, to) = if reverse {
                    (&file_path, &file)
                } else {
                    (&file, &file_path)
                };
                if files_differ(from, to) {
                    info!("Copying file {} to {}", from.display(), to.display());
                    if let Err(err) = std::fs::copy(&from, &to) {
                        return Err(
                            format!("Error copying file {}: {}", file.display(), err).into()
                        );
                    }
                } else {
                    info!(
                        "Skipping copy unchanged file {} to {}",
                        from.display(),
                        to.display()
                    );
                }
            }
        }
    }
    Ok((
        track_copy_dest.values().cloned().collect(),
        track_copy_dest.keys().cloned().collect(),
    ))
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
