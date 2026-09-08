use std::{
    collections::HashMap,
    convert::Infallible,
    sync::{Arc, OnceLock},
};

use axum::{
    Json, Router,
    extract::{Path, State, rejection::JsonRejection},
    http::StatusCode,
    response::{
        IntoResponse, Response, Sse,
        sse::{Event, KeepAlive},
    },
    routing::{get, post},
};
use serde::Serialize;
use tokio::sync::{RwLock, watch};
use tokio_stream::{Stream, StreamExt, wrappers::WatchStream};
use uuid::Uuid;

use crate::{
    methods::{
        EvaluationReport, FittingState, LabeledDocument, SplitStrategy, Trainer, TrainingOutcome,
        TrainingProgress,
        classifiers::{Classifier, NearestCentroid},
    },
    service::Service,
};

mod methods;
pub mod request;
use request::{ClassifierType, CreateModelRequest, SplitSettings, TfIdfSettings, TrainingDocument};

#[derive(Clone, Debug, Serialize)]
pub struct ModelMetadata {
    pub model_id: Uuid,
    pub classifier: ClassifierType,
    pub seed: u64,
    pub feature_count: Option<usize>,
    pub split: SplitSettings,
    pub tf_idf: TfIdfSettings,
    pub progress: TrainingProgress,
    pub evaluation: Option<EvaluationReport>,
    pub error: Option<String>,
}

struct StoredModel {
    metadata: watch::Receiver<ModelMetadata>,
    outcome: OnceLock<Arc<TrainingOutcome<NearestCentroid>>>,
}

#[derive(Clone, Default)]
pub struct ClassificationService {
    models: Arc<RwLock<HashMap<Uuid, Arc<StoredModel>>>>,
}

impl Service for ClassificationService {
    fn router(self) -> Router {
        Router::new()
            .route(Self::BASE_PATH, post(Self::create_model))
            .route("/models/{model_id}", get(Self::model_metadata))
            .route("/models/{model_id}/events", get(Self::model_events))
            .with_state(self)
    }
}

#[derive(Serialize)]
struct AcceptedModel {
    model_id: Uuid,
    seed: u64,
    status_url: String,
    events_url: String,
}

struct ApiError(StatusCode, String);
impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (self.0, Json(serde_json::json!({ "error": self.1 }))).into_response()
    }
}

impl ClassificationService {
    pub const BASE_PATH: &'static str = "/models";

    /// Returns the selected classifier/TF-IDF pair once model creation succeeds.
    pub async fn model(&self, model_id: Uuid) -> Option<Arc<TrainingOutcome<NearestCentroid>>> {
        self.models
            .read()
            .await
            .get(&model_id)
            .and_then(|model| model.outcome.get().cloned())
    }

    async fn create_model(
        State(service): State<Self>,
        input: Result<Json<CreateModelRequest>, JsonRejection>,
    ) -> Result<impl IntoResponse, ApiError> {
        let Json(request) =
            input.map_err(|error| ApiError(StatusCode::BAD_REQUEST, error.body_text()))?;
        request
            .validate()
            .map_err(|error| ApiError(StatusCode::UNPROCESSABLE_ENTITY, error.to_string()))?;
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
        service
            .models
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
        Ok((
            StatusCode::ACCEPTED,
            Json(AcceptedModel {
                model_id,
                seed,
                status_url: format!("/models/{model_id}"),
                events_url: format!("/models/{model_id}/events"),
            }),
        ))
    }

    async fn receiver(&self, id: Uuid) -> Result<watch::Receiver<ModelMetadata>, ApiError> {
        self.models
            .read()
            .await
            .get(&id)
            .map(|model| model.metadata.clone())
            .ok_or_else(|| ApiError(StatusCode::NOT_FOUND, "unknown model".into()))
    }

    async fn model_metadata(
        State(service): State<Self>,
        Path(model_id): Path<Uuid>,
    ) -> Result<Json<ModelMetadata>, ApiError> {
        let receiver = service.receiver(model_id).await?;
        let metadata = receiver.borrow().clone();
        Ok(Json(metadata))
    }

    async fn model_events(
        State(service): State<Self>,
        Path(model_id): Path<Uuid>,
    ) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, ApiError> {
        let stream = WatchStream::new(service.receiver(model_id).await?).map(|metadata| {
            let event = match metadata.progress.state {
                FittingState::Completed => "completed",
                FittingState::Failed => "failed",
                _ => "progress",
            };
            Ok(Event::default()
                .event(event)
                .json_data(metadata)
                .expect("model progress and metrics are finite"))
        });
        Ok(Sse::new(stream).keep_alive(KeepAlive::default()))
    }
}

#[cfg(test)]
mod tests;
