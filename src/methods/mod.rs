use anyhow::{Result, anyhow};
use sprs::CsMat;

use classifiers::Classifier;
use preprocessing::tfidf::{TfIdf, TfIdfBuilder};

pub mod classifiers;
pub mod evaluation;
pub mod preprocessing;
mod split;

pub use evaluation::{EvaluationReport, EvaluationStrategy};
pub use preprocessing::LabeledDocument;
pub(crate) use split::validate_training_documents;
pub use split::{SamplingStrategy, SplitStrategy, TrainTestSplit};

/// Receives only training documents and their TF-IDF rows, in matching order.
pub type ModelCreatingFunction<T, C> = Box<dyn FnMut(Vec<T>, &CsMat<f64>) -> Result<C> + Send>;

pub struct ModelTrainer<T: LabeledDocument, C: Classifier> {
    split_ratio: f32,
    split_strategy: SplitStrategy,
    sampling: SamplingStrategy,
    seed: Option<u64>,
    evaluation_strategy: EvaluationStrategy,
    documents: Vec<T>,
    builder: Option<TfIdfBuilder>,
    method: ModelCreatingFunction<T, C>,
}

impl<T: LabeledDocument, C: Classifier> Default for ModelTrainer<T, C> {
    fn default() -> Self {
        Self {
            split_ratio: 0.8,
            split_strategy: SplitStrategy::ONCE,
            sampling: SamplingStrategy::default(),
            seed: None,
            evaluation_strategy: EvaluationStrategy::default(),
            documents: Vec::new(),
            builder: None,
            method: Box::new(|_, _| Err(anyhow!("Missing model generating function!"))),
        }
    }
}

impl<T: LabeledDocument, C: Classifier> ModelTrainer<T, C> {
    pub fn split_ratio(&mut self, ratio: f32) -> &mut Self {
        self.split_ratio = ratio;
        self
    }
    pub fn train_test_splits(&mut self, strategy: SplitStrategy) -> &mut Self {
        self.split_strategy = strategy;
        self
    }
    pub fn sampling(&mut self, sampling: SamplingStrategy) -> &mut Self {
        self.sampling = sampling;
        self
    }
    pub fn seed(&mut self, seed: u64) -> &mut Self {
        self.seed = Some(seed);
        self
    }
    pub fn evaluation_strategy(&mut self, strategy: EvaluationStrategy) -> &mut Self {
        self.evaluation_strategy = strategy;
        self
    }
    pub fn documents(&mut self, documents: Vec<T>) -> &mut Self {
        self.documents = documents;
        self
    }
    /// Defaults to the existing TF-IDF builder's settings.
    pub fn tf_idf_builder(&mut self, builder: TfIdfBuilder) -> &mut Self {
        self.builder = Some(builder);
        self
    }
    pub fn method(&mut self, model_creator: ModelCreatingFunction<T, C>) -> &mut Self {
        self.method = model_creator;
        self
    }
    /// Train explicitly, returning the selected model and its exact feature space.
    pub fn train(&mut self) -> Result<TrainingOutcome<C>> {
        let seed = *self.seed.get_or_insert_with(rand::random);
        let splits = split::evaluated_splits(
            &self.documents,
            self.split_ratio,
            self.split_strategy,
            self.sampling,
            seed,
        )?;
        let builder = self.builder.get_or_insert_with(TfIdfBuilder::default);
        let mut candidates = Vec::with_capacity(splits.len());
        let mut evaluations = Vec::with_capacity(splits.len());
        for (index, split) in splits.iter().enumerate() {
            let train_documents: Vec<T> = split
                .train_documents
                .iter()
                .map(|&doc| doc.clone())
                .collect();
            let test_documents: Vec<T> = split
                .test_documents
                .iter()
                .map(|&doc| doc.clone())
                .collect();
            let mut tf_idf = builder.build()?;
            let train_features = tf_idf.fit_transform(&train_documents)?;
            let train_labels: Vec<_> = train_documents
                .iter()
                .map(LabeledDocument::label_id)
                .collect();
            let test_labels: Vec<_> = test_documents
                .iter()
                .map(LabeledDocument::label_id)
                .collect();
            let model = (self.method)(train_documents, &train_features)?;
            let test_features = tf_idf.transform(&test_documents)?;
            evaluations.push(evaluation::SplitEvaluation::new(
                index,
                train_labels.len(),
                test_labels.len(),
                evaluation::Metrics::calculate(&train_labels, &model.predict(&train_features))?,
                evaluation::Metrics::calculate(&test_labels, &model.predict(&test_features))?,
            ));
            candidates.push((model, tf_idf));
        }
        let evaluation = EvaluationReport::new(self.evaluation_strategy, seed, evaluations);
        let (model, tf_idf) = candidates.swap_remove(evaluation.selected_split);
        Ok(TrainingOutcome {
            model,
            tf_idf,
            evaluation,
        })
    }
}

/// Owns the selected classifier and its exact feature space, independent of the trainer.
pub struct TrainingOutcome<C: Classifier> {
    pub model: C,
    pub tf_idf: TfIdf,
    pub evaluation: EvaluationReport,
}

#[cfg(test)]
mod tests;
