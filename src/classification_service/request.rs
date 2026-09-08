use anyhow::Result;
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
#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ClassifierType {
    NearestCentroid,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TrainingDocument {
    pub name: String,
    pub pages: Vec<String>,
    pub label: f64,
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
    fn label(&self) -> f64 {
        self.label
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct SplitSettings {
    pub ratio: f32,
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

#[derive(Clone, Debug, Deserialize, Serialize)]
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
    pub(super) fn builder(&self) -> TfIdfBuilder {
        let mut builder = TfIdf::Builder();
        builder
            .tf_weight_scheme(self.tf_scheme)
            .idf_weight_scheme(self.idf_scheme)
            .double_normalization_k(self.double_normalization_k);
        builder
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateModelRequest {
    pub classifier: ClassifierType,
    pub documents: Vec<TrainingDocument>,
    #[serde(default)]
    pub split: SplitSettings,
    #[serde(default)]
    pub tf_idf: TfIdfSettings,
    #[serde(default)]
    pub evaluation_strategy: EvaluationStrategy,
    pub seed: Option<u64>,
}

impl CreateModelRequest {
    pub(super) fn validate(&self) -> Result<()> {
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
