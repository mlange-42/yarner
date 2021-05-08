use std::error::Error;
use std::fmt::{Display, Error as FmtError, Formatter};

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
    Self::IntoIter: Clone,
{
    fn join<S, Q>(self, separator: S, quote: Q) -> Joined<Self::IntoIter, S, Q>
    where
        S: Display,
        Q: Display,
    {
        Joined {
            iter: self.into_iter(),
            separator,
            quote,
        }
    }
}

impl<I> JoinExt for I
where
    I: IntoIterator,
    I::Item: Display,
    I::IntoIter: Clone,
{
}

pub struct Joined<I, S, Q> {
    iter: I,
    separator: S,
    quote: Q,
}

impl<I, S, Q> Display for Joined<I, S, Q>
where
    I: Iterator + Clone,
    I::Item: Display,
    S: Display,
    Q: Display,
{
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result<(), FmtError> {
        let mut iter = self.iter.clone();

        if let Some(val) = iter.next() {
            write!(fmt, "{}{}{}", self.quote, val, self.quote)?;
        }

        for val in iter {
            write!(fmt, "{}{}{}{}", self.separator, self.quote, val, self.quote)?;
        }

        Ok(())
    }
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
        assert_eq!(vec!["foo"].join('\n', "").to_string(), "foo");
        assert_eq!(
            vec!["foo", "bar"].join(", ", '"').to_string(),
            "\"foo\", \"bar\""
        );
        assert_eq!(
            vec!["foo", "bar", "baz"].join('/', '|').to_string(),
            "|foo|/|bar|/|baz|"
        );
    }
}
