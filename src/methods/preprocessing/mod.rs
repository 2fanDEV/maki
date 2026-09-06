use std::future::Future;

use anyhow::Result;
pub mod tfidf;
pub mod util;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FittingState {
    Initialized,
    Queued,
    Running,
    Completed,
    Failed,
}

pub trait FittingTask: Send {
    type Output: Send;

    fn state(&self) -> FittingState;

    fn finish(self) -> impl Future<Output = Result<Self::Output>> + Send
    where
        Self: Sized;
}

pub trait Document: Sync {
    fn name(&self) -> &str;
    fn pages(&self) -> &[String];
    fn pages_size(&self) -> i16;
}
