use serde::Serialize;

use super::preprocessing::FittingState;

/// Percentage of the trainer's complete split collection successfully trained.
/// Evaluation may still be running when training reaches 100%.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct TrainingProgress {
    pub state: FittingState,
    pub trained_splits: usize,
    pub total_splits: usize,
    pub percentage: f64,
}

impl Default for TrainingProgress {
    fn default() -> Self {
        Self {
            state: FittingState::Initialized,
            trained_splits: 0,
            total_splits: 0,
            percentage: 0.0,
        }
    }
}
