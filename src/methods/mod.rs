use anyhow::{Result, anyhow};
use sprs::CsMat;

use classifiers::Classifier;
use preprocessing::tfidf::{TfIdf, TfIdfBuilder};

pub mod classifiers;
pub mod evaluation;
pub mod preprocessing;
mod progress;
mod split;

pub use evaluation::{EvaluationReport, EvaluationStrategy};
pub use preprocessing::{FittingState, LabeledDocument};
pub use progress::TrainingProgress;
pub(crate) use split::validate_training_documents;
pub use split::{SamplingStrategy, SplitStrategy, TrainTestSplit};

/// Receives only training documents and their TF-IDF rows, in matching order.
pub type ModelCreatingFunction<T, C> = Box<dyn FnMut(Vec<T>, &CsMat<f64>) -> Result<C>>;

pub struct TrainerBuilder<T: LabeledDocument, C: Classifier> {
    split_ratio: f32,
    split_strategy: SplitStrategy,
    sampling: SamplingStrategy,
    seed: Option<u64>,
    evaluation_strategy: EvaluationStrategy,
    documents: Vec<T>,
    builder: Option<TfIdfBuilder>,
    method: ModelCreatingFunction<T, C>,
    observer: Box<dyn FnMut(TrainingProgress)>,
}

impl<T: LabeledDocument, C: Classifier> Default for TrainerBuilder<T, C> {
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
            observer: Box::new(|_| {}),
        }
    }
}

impl<T: LabeledDocument, C: Classifier> TrainerBuilder<T, C> {
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
    pub fn progress_observer(
        &mut self,
        observer: impl FnMut(TrainingProgress) + 'static,
    ) -> &mut Self {
        self.observer = Box::new(observer);
        self
    }
    /// Prepare all splits and pass them to the trainer for training and evaluation.
    pub fn build(&mut self) -> Result<Trainer<'_, T, C>> {
        let seed = self.seed.unwrap_or_else(rand::random);
        let splits = split::evaluated_splits(
            &self.documents,
            self.split_ratio,
            self.split_strategy,
            self.sampling,
            seed,
        )?;
        let builder = self.builder.get_or_insert_with(TfIdfBuilder::default);
        Trainer::new(
            splits,
            builder,
            &mut self.method,
            &mut self.observer,
            self.evaluation_strategy,
            seed,
        )
    }
}

pub struct Trainer<'a, T: LabeledDocument, C: Classifier> {
    tf_idfs: Vec<TfIdf>,
    train_test_documents: Vec<TrainTestSplit<'a, T>>,
    models: Vec<C>,
    evaluation: Option<EvaluationReport>,
    progress: TrainingProgress,
}

/// Owns the selected model and its exact feature space; independent of the builder.
pub struct TrainingOutcome<C: Classifier> {
    pub model: C,
    pub tf_idf: TfIdf,
    pub evaluation: EvaluationReport,
    pub progress: TrainingProgress,
}

impl<'a, T: LabeledDocument, C: Classifier> Trainer<'a, T, C> {
    #[allow(non_snake_case)]
    pub fn Builder() -> TrainerBuilder<T, C> {
        TrainerBuilder::default()
    }

    /// Models, TF-IDF instances, splits, and reports share the same index.
    pub fn models(&self) -> &[C] {
        &self.models
    }
    pub fn tf_idfs(&self) -> &[TfIdf] {
        &self.tf_idfs
    }
    pub fn train_test_documents(&self) -> &[TrainTestSplit<'a, T>] {
        &self.train_test_documents
    }
    pub fn evaluation(&self) -> &EvaluationReport {
        self.evaluation
            .as_ref()
            .expect("a constructed trainer has completed evaluation")
    }
    pub fn progress(&self) -> TrainingProgress {
        self.progress
    }

    pub fn into_best(mut self) -> TrainingOutcome<C> {
        let index = self.evaluation().selected_split;
        TrainingOutcome {
            model: self.models.swap_remove(index),
            tf_idf: self.tf_idfs.swap_remove(index),
            evaluation: self
                .evaluation
                .take()
                .expect("a constructed trainer has completed evaluation"),
            progress: self.progress,
        }
    }

    fn new(
        train_test_documents: Vec<TrainTestSplit<'a, T>>,
        builder: &TfIdfBuilder,
        model_creator: &mut ModelCreatingFunction<T, C>,
        observer: &mut dyn FnMut(TrainingProgress),
        strategy: EvaluationStrategy,
        seed: u64,
    ) -> Result<Self> {
        let total_splits = train_test_documents.len();
        crate::invariant!(total_splits > 0, "a trainer needs at least one split");
        let mut trainer = Self {
            tf_idfs: Vec::with_capacity(total_splits),
            models: Vec::with_capacity(total_splits),
            train_test_documents,
            evaluation: None,
            progress: TrainingProgress {
                state: FittingState::Running,
                total_splits,
                ..TrainingProgress::default()
            },
        };
        observer(trainer.progress());
        let result = trainer.train_and_evaluate(builder, model_creator, observer, strategy, seed);
        trainer.progress.state = if result.is_ok() {
            FittingState::Completed
        } else {
            FittingState::Failed
        };
        observer(trainer.progress());
        result?;
        Ok(trainer)
    }

    fn train_and_evaluate(
        &mut self,
        builder: &TfIdfBuilder,
        model_creator: &mut ModelCreatingFunction<T, C>,
        observer: &mut dyn FnMut(TrainingProgress),
        strategy: EvaluationStrategy,
        seed: u64,
    ) -> Result<()> {
        let mut evaluations = Vec::with_capacity(self.train_test_documents.len());
        for index in 0..self.train_test_documents.len() {
            let split = &self.train_test_documents[index];
            crate::invariant!(
                !split.train_documents.is_empty() && !split.test_documents.is_empty(),
                "training and test partitions must be nonempty"
            );
            let documents: Vec<T> = split
                .train_documents
                .iter()
                .map(|&doc| doc.clone())
                .collect();
            let mut tf_idf = builder.build()?;
            let train_features = tf_idf.fit_transform(&documents)?;
            let model = model_creator(documents, &train_features)?;
            self.tf_idfs.push(tf_idf);
            self.models.push(model);

            // Only the trainer knows the full denominator. A failed fit never advances it.
            self.progress.trained_splits = self.models.len();
            self.progress.percentage = self.progress.trained_splits as f64 * 100.0
                / self.train_test_documents.len() as f64;
            observer(self.progress());
            evaluations.push(self.evaluate_split(index, &train_features)?);
        }
        self.evaluation = Some(EvaluationReport::new(strategy, seed, evaluations));
        Ok(())
    }

    fn evaluate_split(
        &self,
        index: usize,
        train_features: &CsMat<f64>,
    ) -> Result<evaluation::SplitEvaluation> {
        let split = &self.train_test_documents[index];
        let documents: Vec<T> = split
            .test_documents
            .iter()
            .map(|&doc| doc.clone())
            .collect();
        let test_features = self.tf_idfs[index].transform(&documents)?;
        let model = &self.models[index];
        let train_labels: Vec<_> = split
            .train_documents
            .iter()
            .map(|doc| doc.label())
            .collect();
        let test_labels: Vec<_> = split.test_documents.iter().map(|doc| doc.label()).collect();
        Ok(evaluation::SplitEvaluation::new(
            index,
            train_labels.len(),
            test_labels.len(),
            evaluation::Metrics::calculate(&train_labels, &model.predict(train_features))?,
            evaluation::Metrics::calculate(&test_labels, &model.predict(&test_features))?,
        ))
    }
}

#[cfg(test)]
mod tests;
