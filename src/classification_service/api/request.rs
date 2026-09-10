use anyhow::Result;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

use crate::{
    invariant,
    methods::{
        EvaluationStrategy, LabeledDocument, SamplingStrategy, SplitStrategy,
        preprocessing::{
            Document,
            tfidf::{IdfWeightScheme, TfIdf, TfIdfBuilder, TfWeightScheme},
        },
        validate_training_documents,
    },
};

/// Supported model types. Add variants and their training dispatch together.
#[derive(utoipa::ToSchema, Clone, Copy, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ClassifierType {
    NearestCentroid,
}

#[derive(utoipa::ToSchema, Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TrainingDocument {
    pub name: String,
    pub pages: Vec<String>,
    /// Hexadecimal MongoDB label ID. Each class needs at least two documents.
    #[serde(deserialize_with = "deserialize_label_id")]
    #[schema(value_type = String)]
    pub label_id: ObjectId,
}

impl Document for TrainingDocument {
    fn name(&self) -> &str {
        &self.name
    }
    fn pages(&self) -> &[String] {
        &self.pages
    }
    fn pages_size(&self) -> i16 {
        self.pages.len() as i16
    }
}
impl LabeledDocument for TrainingDocument {
    fn label_id(&self) -> ObjectId {
        self.label_id
    }
}

#[derive(utoipa::ToSchema, Clone, Debug, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct SplitSettings {
    /// Training fraction, strictly between zero and one.
    pub ratio: f32,
    /// Number of train/test splits; must be at least one.
    pub repetitions: usize,
    pub sampling: SamplingStrategy,
}
impl Default for SplitSettings {
    fn default() -> Self {
        Self {
            ratio: 0.8,
            repetitions: 5,
            sampling: SamplingStrategy::Stratified,
        }
    }
}

#[derive(utoipa::ToSchema, Clone, Debug, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct TfIdfSettings {
    pub tf_scheme: TfWeightScheme,
    pub idf_scheme: IdfWeightScheme,
    pub double_normalization_k: f64,
}
impl Default for TfIdfSettings {
    fn default() -> Self {
        Self {
            tf_scheme: TfWeightScheme::default(),
            idf_scheme: IdfWeightScheme::default(),
            double_normalization_k: 0.5,
        }
    }
}
impl TfIdfSettings {
    pub(in crate::classification_service) fn builder(&self) -> TfIdfBuilder {
        let mut builder = TfIdf::Builder();
        builder
            .tf_weight_scheme(self.tf_scheme)
            .idf_weight_scheme(self.idf_scheme)
            .double_normalization_k(self.double_normalization_k);
        builder
    }
}

#[derive(utoipa::ToSchema, Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateTrainerRequest {
    pub classifier: ClassifierType,
    pub documents: Vec<TrainingDocument>,
    #[serde(default)]
    pub split: SplitSettings,
    #[serde(default)]
    pub tf_idf: TfIdfSettings,
    #[serde(default)]
    pub evaluation_strategy: EvaluationStrategy,
    /// Random seed for reproducible splits. Generated and returned when omitted.
    pub seed: Option<u64>,
}

impl CreateTrainerRequest {
    pub(in crate::classification_service) fn validate(&self) -> Result<()> {
        validate_training_documents(
            &self.documents,
            self.split.ratio,
            SplitStrategy::REPEATED(self.split.repetitions),
            self.split.sampling,
        )?;
        invariant!(
            self.documents
                .iter()
                .all(|document| document.pages.len() <= i16::MAX as usize),
            "document page count exceeds the Document interface limit"
        );
        self.tf_idf.builder().build()?;
        Ok(())
    }
}

#[derive(Deserialize, utoipa::ToSchema)]
pub(super) struct ModelPath {
    pub(super) model_id: uuid::Uuid,
}

#[derive(Deserialize, utoipa::ToSchema)]
pub(super) struct TrainerPath {
    pub(super) trainer_id: uuid::Uuid,
}

fn deserialize_label_id<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<ObjectId, D::Error> {
    let id = String::deserialize(deserializer)?;
    ObjectId::parse_str(id).map_err(serde::de::Error::custom)
}
