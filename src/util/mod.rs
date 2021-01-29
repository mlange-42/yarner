use std::error::Error;

pub mod try_collect;

pub type Fallible<T = ()> = Result<T, Box<dyn Error>>;
