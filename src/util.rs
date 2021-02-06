use std::error::Error;
use std::ffi::OsStr;
use std::fmt::{Display, Write};
use std::iter::repeat;
use std::path::{Path, PathBuf};

pub type Fallible<T = ()> = Result<T, Box<dyn Error>>;

pub trait TryCollectExt<T, E>
where
    Self: Iterator<Item = Result<T, E>> + Sized,
{
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

pub trait JoinExt
where
    Self: IntoIterator + Sized,
    Self::Item: Display,
{
    fn join<S, Q>(self, separator: S, quote: Q) -> String
    where
        S: Display,
        Q: Display,
    {
        let mut joined = String::new();
        let mut iter = self.into_iter();

        if let Some(val) = iter.next() {
            write!(&mut joined, "{}{}{}", quote, val, quote).unwrap();
        }

        for val in iter {
            write!(&mut joined, "{}{}{}{}", separator, quote, val, quote).unwrap();
        }

        joined
    }
}

impl<I> JoinExt for I
where
    I: IntoIterator,
    I::Item: Display,
{
}

pub trait PushSimple<P> {
    fn push_simple(&mut self, other: P);
}

impl PushSimple<PathBuf> for PathBuf {
    fn push_simple(&mut self, other: PathBuf) {
        for comp in other.components() {
            match comp.as_os_str().to_str() {
                None => {}
                Some(comp) => {
                    if comp == ".." {
                        if !self.pop() {
                            self.push(comp);
                        }
                    } else {
                        self.push(comp);
                    }
                }
            }
        }
    }
}

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
    fn collect_vals() {
        assert_eq!(
            vec![Ok(1), Ok(2), Ok(3)].into_iter().try_collect(),
            Ok::<_, Vec<()>>(vec![1, 2, 3])
        );
    }

    #[test]
    fn collect_errs() {
        assert_eq!(
            vec![Ok(1), Err(2), Ok(3)].into_iter().try_collect(),
            Err(vec![2])
        );

        assert_eq!(
            vec![Ok(1), Err(2), Err(3)].into_iter().try_collect(),
            Err(vec![2, 3])
        );
    }

    #[test]
    fn join_strs() {
        assert_eq!(vec!["foo"].join('\n', ""), "foo");
        assert_eq!(vec!["foo", "bar"].join(", ", '"'), "\"foo\", \"bar\"");
        assert_eq!(
            vec!["foo", "bar", "baz"].join('/', '|'),
            "|foo|/|bar|/|baz|"
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

    #[test]
    fn push_simple() {
        let mut path1 = PathBuf::from("");
        path1.push_simple(PathBuf::from("bar"));
        assert_eq!(path1, PathBuf::from("bar"));

        let mut path1 = PathBuf::from("foo");
        path1.push_simple(PathBuf::from(""));
        assert_eq!(path1, PathBuf::from("foo"));

        let mut path1 = PathBuf::from("foo");
        path1.push_simple(PathBuf::from("bar"));
        assert_eq!(path1, PathBuf::from("foo/bar"));

        let mut path1 = PathBuf::from("foo");
        path1.push_simple(PathBuf::from("../bar"));
        assert_eq!(path1, PathBuf::from("bar"));

        let mut path1 = PathBuf::from("foo");
        path1.push_simple(PathBuf::from(".."));
        assert_eq!(path1, PathBuf::from(""));

        let mut path1 = PathBuf::from("foo");
        path1.push_simple(PathBuf::from("bar/../baz"));
        assert_eq!(path1, PathBuf::from("foo/baz"));
    }
}
