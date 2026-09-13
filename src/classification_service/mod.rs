use std::{collections::HashMap, sync::Arc};

use tokio::sync::Mutex;
use uuid::Uuid;

use crate::{
    label_service::LabelService,
    methods::{
        LabeledDocument, ModelTrainer, SplitStrategy, TrainingOutcome,
        classifiers::{Classifier, NearestCentroid},
    },
    service::ServiceError,
};
use api::{
    request::{ClassifierType, CreateTrainerRequest, TrainingDocument},
    response::{ModelMetadata, TrainerMetadata, TrainerStatus},
};

pub mod api;
mod methods;

type Outcome = TrainingOutcome<NearestCentroid>;

struct StoredTrainer {
    metadata: TrainerMetadata,
    trainer: Option<ModelTrainer<TrainingDocument, NearestCentroid>>,
}

struct StoredModel {
    metadata: ModelMetadata,
    outcome: Arc<Outcome>,
}

#[derive(Default)]
struct Store {
    trainers: HashMap<Uuid, StoredTrainer>,
    models: HashMap<Uuid, StoredModel>,
}

#[derive(Clone, Default)]
pub struct ClassificationService {
    store: Arc<Mutex<Store>>,
}

impl ClassificationService {
    /// Returns the selected classifier/TF-IDF pair for a completed model.
    pub async fn model(&self, model_id: Uuid) -> Option<Arc<Outcome>> {
        self.store
            .lock()
            .await
            .models
            .get(&model_id)
            .map(|model| Arc::clone(&model.outcome))
    }

    async fn create_trainer(
        &self,
        request: CreateTrainerRequest,
        labels: &LabelService,
    ) -> Result<TrainerMetadata, ServiceError> {
        let ids: Vec<_> = request
            .documents
            .iter()
            .map(LabeledDocument::label_id)
            .collect();
        let trainer = Self::prepare_trainer(request)?;
        labels.validate_ids(&ids).await?;
        Ok(self.store_trainer(trainer).await)
    }

    fn prepare_trainer(request: CreateTrainerRequest) -> Result<StoredTrainer, ServiceError> {
        request
            .validate()
            .map_err(|error| ServiceError::InvalidInput(error.to_string()))?;
        let seed = request.seed.unwrap_or_else(rand::random);
        let mut label_ids: Vec<_> = request
            .documents
            .iter()
            .map(LabeledDocument::label_id)
            .collect();
        label_ids.sort_unstable();
        label_ids.dedup();
        let metadata = TrainerMetadata {
            trainer_id: Uuid::new_v4(),
            classifier: request.classifier,
            seed,
            label_ids: label_ids.into_iter().map(|id| id.to_hex()).collect(),
            split: request.split.clone(),
            tf_idf: request.tf_idf.clone(),
            evaluation_strategy: request.evaluation_strategy,
            status: TrainerStatus::Created,
            model_id: None,
            error: None,
        };
        let mut trainer = ModelTrainer::default();
        trainer
            .documents(request.documents)
            .split_ratio(request.split.ratio)
            .train_test_splits(SplitStrategy::REPEATED(request.split.repetitions))
            .sampling(request.split.sampling)
            .seed(seed)
            .evaluation_strategy(request.evaluation_strategy)
            .tf_idf_builder(request.tf_idf.builder());
        match request.classifier {
            ClassifierType::NearestCentroid => {
                trainer.method(Box::new(|documents, features| {
                    let labels: Vec<_> = documents.iter().map(LabeledDocument::label_id).collect();
                    NearestCentroid::train(features, &labels)
                }));
            }
        }
        Ok(StoredTrainer {
            metadata,
            trainer: Some(trainer),
        })
    }

    async fn store_trainer(&self, trainer: StoredTrainer) -> TrainerMetadata {
        let metadata = trainer.metadata.clone();
        self.store
            .lock()
            .await
            .trainers
            .insert(metadata.trainer_id, trainer);
        metadata
    }

    async fn trainer_metadata(&self, id: Uuid) -> Result<TrainerMetadata, ServiceError> {
        self.store
            .lock()
            .await
            .trainers
            .get(&id)
            .map(|trainer| trainer.metadata.clone())
            .ok_or(ServiceError::NotFound("unknown trainer"))
    }

    async fn model_metadata(&self, id: Uuid) -> Result<ModelMetadata, ServiceError> {
        self.store
            .lock()
            .await
            .models
            .get(&id)
            .map(|model| model.metadata.clone())
            .ok_or(ServiceError::NotFound("unknown model"))
    }

    async fn train(&self, trainer_id: Uuid) -> Result<ModelMetadata, ServiceError> {
        let mut store = self.store.lock().await;
        let stored = store
            .trainers
            .get_mut(&trainer_id)
            .ok_or(ServiceError::NotFound("unknown trainer"))?;
        let mut trainer = stored
            .trainer
            .take()
            .ok_or(ServiceError::Conflict("trainer has already been started"))?;
        stored.metadata.status = TrainerStatus::Training;
        let metadata = stored.metadata.clone();
        let service = self.clone();
        // This task owns both fitting and publication. Dropping the HTTP future does not cancel it.
        let worker = tokio::spawn(async move {
            let result = tokio::task::spawn_blocking(move || trainer.train()).await;
            let outcome = match result {
                Ok(Ok(outcome)) => outcome,
                Ok(Err(error)) => {
                    let message = error.to_string();
                    service.fail_trainer(trainer_id, message.clone()).await;
                    return Err(ServiceError::InvalidInput(message));
                }
                Err(error) => {
                    service
                        .fail_trainer(trainer_id, "training worker failed".into())
                        .await;
                    return Err(ServiceError::Internal(error.into()));
                }
            };
            let model = ModelMetadata {
                model_id: Uuid::new_v4(),
                trainer_id,
                classifier: metadata.classifier,
                seed: metadata.seed,
                label_ids: outcome
                    .model
                    .labels()
                    .into_iter()
                    .map(|id| id.to_hex())
                    .collect(),
                feature_count: outcome
                    .tf_idf
                    .vocabulary()
                    .expect("trained TF-IDF has a vocabulary")
                    .len(),
                split: metadata.split,
                tf_idf: metadata.tf_idf,
                evaluation: outcome.evaluation.clone(),
            };
            let mut store = service.store.lock().await;
            store.models.insert(
                model.model_id,
                StoredModel {
                    metadata: model.clone(),
                    outcome: Arc::new(outcome),
                },
            );
            let trainer = store
                .trainers
                .get_mut(&trainer_id)
                .expect("trainer remains registered");
            trainer.metadata.status = TrainerStatus::Completed;
            trainer.metadata.model_id = Some(model.model_id);
            Ok(model)
        });
        // Launch the owning task before releasing the lock or reaching a cancellation point.
        drop(store);
        worker
            .await
            .map_err(|error| ServiceError::Internal(error.into()))?
    }

    async fn fail_trainer(&self, id: Uuid, message: String) {
        let mut store = self.store.lock().await;
        let trainer = store
            .trainers
            .get_mut(&id)
            .expect("trainer remains registered");
        trainer.metadata.status = TrainerStatus::Failed;
        trainer.metadata.error = Some(message);
    }
}

#[cfg(test)]
mod tests;
