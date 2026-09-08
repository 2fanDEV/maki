use std::future::Future;

use anyhow::Result;
pub mod tfidf;
pub mod util;

#[derive(schemars::JsonSchema, Clone, Copy, Debug, Eq, PartialEq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
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

pub trait Document: Sync + Clone {
    fn name(&self) -> &str;
    fn pages(&self) -> &[String];
    fn pages_size(&self) -> i16;
}

/// A classification document with a finite numeric class label.
pub trait LabeledDocument: Document {
    fn label(&self) -> f64;
}
