use std::{fmt::Debug, time::Duration};

pub trait BackoffBuilder: Clone + Debug + Send + Sync + Unpin {
    type Backoff: Backoff;
    fn build(&self) -> Self::Backoff;
}

pub trait Backoff: Iterator<Item = Duration> + Send + Sync + Unpin {}
impl<T> Backoff for T where T: Iterator<Item = Duration> + Send + Sync + Unpin {}
