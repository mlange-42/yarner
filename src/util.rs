use std::error::Error;
use std::fmt::{Display, Write};

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

#[cfg(test)]
mod tests {
    use super::*;

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
        assert_eq!(vec!["foo"].join("\n", ""), "foo");
        assert_eq!(vec!["foo", "bar"].join(", ", "'"), "'foo', 'bar'");
        assert_eq!(
            vec!["foo", "bar", "baz"].join("/", "|"),
            "|foo|/|bar|/|baz|"
        );
    }
}
