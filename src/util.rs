use std::error::Error;

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
