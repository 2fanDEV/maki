use serde::Serialize;
use uuid::Uuid;

use super::request::{ClassifierType, SplitSettings, TfIdfSettings};
use crate::methods::{EvaluationReport, EvaluationStrategy};

#[derive(schemars::JsonSchema, Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TrainerStatus {
    Created,
    Training,
    Completed,
    Failed,
}

#[derive(schemars::JsonSchema, Clone, Debug, Serialize)]
pub struct TrainerMetadata {
    pub trainer_id: Uuid,
    pub classifier: ClassifierType,
    pub seed: u64,
    pub label_ids: Vec<String>,
    pub split: SplitSettings,
    pub tf_idf: TfIdfSettings,
    pub evaluation_strategy: EvaluationStrategy,
    pub status: TrainerStatus,
    pub model_id: Option<Uuid>,
    pub error: Option<String>,
}

#[derive(schemars::JsonSchema, Clone, Debug, Serialize)]
pub struct ModelMetadata {
    pub model_id: Uuid,
    pub trainer_id: Uuid,
    pub classifier: ClassifierType,
    pub seed: u64,
    pub label_ids: Vec<String>,
    pub feature_count: usize,
    pub split: SplitSettings,
    pub tf_idf: TfIdfSettings,
    pub evaluation: EvaluationReport,
}
