use std::iter::FromIterator;

pub(crate) trait TryCollectExt: Iterator + Sized {
    fn try_collect<R, E, Cr, Ce>(self) -> Result<Cr, Ce>
    where
        Self: Iterator<Item = Result<R, E>>,
        Cr: FromIterator<R>,
        Ce: FromIterator<E>,
    {
        let result = self.fold(Ok(vec![]), |collection, node| match node {
            Err(error) => match collection {
                Ok(..) => Err(vec![error]),
                Err(mut errors) => {
                    errors.push(error);
                    Err(errors)
                }
            },
            Ok(node) => collection.map(move |mut nodes| {
                nodes.push(node);
                nodes
            }),
        });
        match result {
            Ok(ok) => Ok(ok.into_iter().collect()),
            Err(err) => Err(err.into_iter().collect()),
        }
    }
}

impl<I: Iterator> TryCollectExt for I {}
