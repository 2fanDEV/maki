use std::{
    collections::HashMap,
    sync::{Arc, OnceLock},
};

use tokio::sync::{RwLock, watch};
use uuid::Uuid;

use crate::methods::{
    FittingState, LabeledDocument, SplitStrategy, Trainer, TrainingOutcome, TrainingProgress,
    classifiers::{Classifier, NearestCentroid},
};
use api::{
    request::{ClassifierType, CreateModelRequest, TrainingDocument},
    response::ModelMetadata,
};

pub mod api;
mod methods;

struct StoredModel {
    metadata: watch::Receiver<ModelMetadata>,
    outcome: OnceLock<Arc<TrainingOutcome<NearestCentroid>>>,
}

#[derive(Clone, Default)]
pub struct ClassificationService {
    models: Arc<RwLock<HashMap<Uuid, Arc<StoredModel>>>>,
}

impl ClassificationService {
    /// Returns the selected classifier/TF-IDF pair once model creation succeeds.
    pub async fn model(&self, model_id: Uuid) -> Option<Arc<TrainingOutcome<NearestCentroid>>> {
        self.models
            .read()
            .await
            .get(&model_id)
            .and_then(|model| model.outcome.get().cloned())
    }

    async fn create_model(&self, request: CreateModelRequest) -> anyhow::Result<(Uuid, u64)> {
        request.validate()?;
        let seed = request.seed.unwrap_or_else(rand::random);
        let model_id = Uuid::new_v4();
        let (updates, receiver) = watch::channel(ModelMetadata {
            model_id,
            classifier: request.classifier,
            seed,
            feature_count: None,
            split: request.split.clone(),
            tf_idf: request.tf_idf.clone(),
            progress: TrainingProgress::default(),
            evaluation: None,
            error: None,
        });
        let model = Arc::new(StoredModel {
            metadata: receiver,
            outcome: OnceLock::new(),
        });
        self.models
            .write()
            .await
            .insert(model_id, Arc::clone(&model));

        // Execute CPU work off the HTTP runtime. The model ID is the only resource ID.
        let progress_updates = updates.clone();
        let training = tokio::task::spawn_blocking(move || {
            let mut builder = Trainer::<TrainingDocument, NearestCentroid>::Builder();
            builder
                .documents(request.documents)
                .split_ratio(request.split.ratio)
                .train_test_splits(SplitStrategy::REPEATED(request.split.repetitions))
                .sampling(request.split.sampling)
                .seed(seed)
                .evaluation_strategy(request.evaluation_strategy)
                .tf_idf_builder(request.tf_idf.builder())
                .progress_observer(move |progress| {
                    // Terminal status is published after the selected model is accessible.
                    // All split counts and percentages come directly from the trainer.
                    if progress.state == FittingState::Running {
                        progress_updates.send_modify(|metadata| metadata.progress = progress);
                    }
                });
            match request.classifier {
                ClassifierType::NearestCentroid => {
                    builder.method(Box::new(|documents, features| {
                        let labels: Vec<_> =
                            documents.iter().map(|document| document.label()).collect();
                        NearestCentroid::train(features, &labels)
                    }));
                }
            }
            Ok::<_, anyhow::Error>(builder.build()?.into_best())
        });
        tokio::spawn(async move {
            let result = training
                .await
                .map_err(anyhow::Error::from)
                .and_then(|result| result);
            match result {
                Ok(outcome) => {
                    let outcome = Arc::new(outcome);
                    let _ = model.outcome.set(Arc::clone(&outcome));
                    updates.send_modify(|metadata| {
                        metadata.feature_count = Some(
                            outcome
                                .tf_idf
                                .vocabulary()
                                .expect("trained TF-IDF has a vocabulary")
                                .len(),
                        );
                        metadata.evaluation = Some(outcome.evaluation.clone());
                        metadata.progress = outcome.progress;
                    });
                }
                Err(error) => {
                    updates.send_modify(|metadata| {
                        // Preserve the trainer's last percentage; never count a failed fit.
                        metadata.progress.state = FittingState::Failed;
                        metadata.error = Some(format!("{error:#}"));
                    });
                }
            }
            // Closing the producer ends SSE after the final model update.
        });
        Ok((model_id, seed))
    }

    async fn receiver(&self, id: Uuid) -> Option<watch::Receiver<ModelMetadata>> {
        self.models
            .read()
            .await
            .get(&id)
            .map(|model| model.metadata.clone())
    }
}

#[cfg(test)]
mod tests;
