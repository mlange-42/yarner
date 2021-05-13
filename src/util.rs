use std::error::Error;

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
}
