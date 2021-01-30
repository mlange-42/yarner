use std::error::Error;
use std::ffi::OsStr;
use std::iter::repeat;
use std::path::{Path, PathBuf};

pub type Fallible<T = ()> = Result<T, Box<dyn Error>>;

pub trait TryCollectExt<T, E>: Iterator<Item = Result<T, E>> + Sized {
    fn try_collect(self) -> Result<Vec<T>, Vec<E>> {
        let vals = Vec::with_capacity(self.size_hint().0);

        self.fold(Ok(vals), |results, result| match (results, result) {
            (Ok(mut vals), Ok(val)) => {
                vals.push(val);
                Ok(vals)
            }
            (Ok(_vals), Err(err)) => Err(vec![err]),
            (Err(errs), Ok(_val)) => Err(errs),
            (Err(mut errs), Err(err)) => {
                errs.push(err);
                Err(errs)
            }
        })
    }
}

impl<I, T, E> TryCollectExt<T, E> for I where I: Iterator<Item = Result<T, E>> {}

pub fn modify_path(path: &Path, replace: &str) -> PathBuf {
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
