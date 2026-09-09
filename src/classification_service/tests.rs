use super::*;
use crate::methods::{
    EvaluationStrategy,
    preprocessing::tfidf::{IdfWeightScheme, TfWeightScheme},
};
use mongodb::bson::oid::ObjectId;

fn label(value: u8) -> ObjectId {
    ObjectId::from_bytes([value; 12])
}

fn request() -> CreateTrainerRequest {
    CreateTrainerRequest {
        classifier: ClassifierType::NearestCentroid,
        documents: (0..10)
            .map(|index| TrainingDocument {
                name: format!("doc-{index}"),
                pages: vec![
                    if index < 5 {
                        "apple fruit sweet"
                    } else {
                        "orange citrus sour"
                    }
                    .into(),
                ],
                label_id: if index < 5 { label(3) } else { label(8) },
            })
            .collect(),
        split: Default::default(),
        tf_idf: Default::default(),
        evaluation_strategy: Default::default(),
        seed: Some(42),
    }
}

// Exercise in-memory lifecycle logic independently of MongoDB label lookup.
async fn register(
    service: &ClassificationService,
    request: CreateTrainerRequest,
) -> TrainerMetadata {
    service
        .store_trainer(ClassificationService::prepare_trainer(request).unwrap())
        .await
}

#[tokio::test]
async fn creating_then_training_publishes_a_usable_separately_identified_model() {
    let service = ClassificationService::default();
    let created = register(&service, request()).await;
    assert_eq!(created.status, TrainerStatus::Created);
    assert_eq!(created.seed, 42);
    assert!(created.model_id.is_none());
    assert!(service.store.lock().await.models.is_empty());
    assert_eq!(
        created.label_ids,
        vec![label(3).to_hex(), label(8).to_hex()]
    );

    // Concurrent starts must not train or publish twice.
    let (first, second) = tokio::join!(
        service.train(created.trainer_id),
        service.train(created.trainer_id)
    );
    let completed = first.unwrap();
    assert!(matches!(second, Err(ServiceError::Conflict(_))));
    assert_ne!(completed.model_id, created.trainer_id);
    assert_eq!(completed.trainer_id, created.trainer_id);
    assert_eq!(completed.evaluation.splits.len(), 5);
    let metadata = service.trainer_metadata(created.trainer_id).await.unwrap();
    assert_eq!(metadata.status, TrainerStatus::Completed);
    assert_eq!(metadata.model_id, Some(completed.model_id));
    assert!(matches!(
        service.train(created.trainer_id).await,
        Err(ServiceError::Conflict(_))
    ));
    assert_eq!(service.store.lock().await.models.len(), 1);
    assert_eq!(
        service
            .model_metadata(completed.model_id)
            .await
            .unwrap()
            .evaluation,
        completed.evaluation
    );

    let outcome = service.model(completed.model_id).await.unwrap();
    let documents = vec![
        TrainingDocument {
            name: "new apple".into(),
            pages: vec!["apple sweet".into()],
            label_id: label(3),
        },
        TrainingDocument {
            name: "new orange".into(),
            pages: vec!["orange".into()],
            label_id: label(8),
        },
    ];
    let features = outcome.tf_idf.transform(&documents).unwrap();
    assert_eq!(outcome.model.predict(&features), [label(3), label(8)]);
    assert_eq!(outcome.model.labels(), [label(3), label(8)]);
}

#[tokio::test]
async fn failed_training_records_an_error_without_publishing_a_model() {
    let service = ClassificationService::default();
    let mut input = request();
    for document in &mut input.documents {
        document.pages = vec![String::new()];
    }
    let created = register(&service, input).await;
    assert!(matches!(
        service.train(created.trainer_id).await,
        Err(ServiceError::InvalidInput(_))
    ));
    let failed = service.trainer_metadata(created.trainer_id).await.unwrap();
    assert_eq!(failed.status, TrainerStatus::Failed);
    assert!(failed.error.unwrap().contains("training data"));
    assert!(failed.model_id.is_none());
    assert!(service.store.lock().await.models.is_empty());
    assert!(matches!(
        service.train(created.trainer_id).await,
        Err(ServiceError::Conflict(_))
    ));
}

#[tokio::test]
async fn unknown_resources_and_invalid_training_settings_are_rejected() {
    let service = ClassificationService::default();
    let id = Uuid::new_v4();
    assert!(matches!(
        service.train(id).await,
        Err(ServiceError::NotFound(_))
    ));
    assert!(matches!(
        service.trainer_metadata(id).await,
        Err(ServiceError::NotFound(_))
    ));
    assert!(matches!(
        service.model_metadata(id).await,
        Err(ServiceError::NotFound(_))
    ));
    assert!(service.model(id).await.is_none());
    for ratio in [0.0, 1.0, f32::NAN] {
        let mut input = request();
        input.split.ratio = ratio;
        assert!(matches!(
            ClassificationService::prepare_trainer(input),
            Err(ServiceError::InvalidInput(_))
        ));
    }
    let mut input = request();
    input.split.repetitions = 0;
    assert!(ClassificationService::prepare_trainer(input).is_err());
    let mut input = request();
    input.documents.clear();
    assert!(ClassificationService::prepare_trainer(input).is_err());
    let mut input = request();
    input.tf_idf.double_normalization_k = 2.0;
    assert!(ClassificationService::prepare_trainer(input).is_err());
}

#[tokio::test]
async fn generated_seed_and_custom_settings_reproduce_evaluation() {
    let service = ClassificationService::default();
    let mut input = request();
    input.seed = None;
    input.evaluation_strategy = EvaluationStrategy::MatthewsCorrelation;
    input.split.ratio = 0.6;
    input.split.repetitions = 2;
    input.tf_idf.idf_scheme = IdfWeightScheme::SMOOTH;
    input.tf_idf.tf_scheme = TfWeightScheme::BINARY;
    // The request owns its documents, so assemble the replay separately.
    let mut replay = request();
    replay.evaluation_strategy = input.evaluation_strategy;
    replay.split = input.split.clone();
    replay.tf_idf = input.tf_idf.clone();
    let first = register(&service, input).await;
    replay.seed = Some(first.seed);
    let second = register(&service, replay).await;
    let a = service.train(first.trainer_id).await.unwrap();
    let b = service.train(second.trainer_id).await.unwrap();
    assert_eq!(a.evaluation, b.evaluation);
    assert_eq!(a.evaluation.seed, first.seed);
    assert_eq!(
        a.evaluation.strategy,
        EvaluationStrategy::MatthewsCorrelation
    );
    assert_eq!(a.evaluation.splits.len(), 2);
    assert_eq!(a.evaluation.splits[0].train_samples, 6);
    let outcome = service.model(a.model_id).await.unwrap();
    assert!(
        outcome
            .tf_idf
            .idf_weights()
            .unwrap()
            .iter()
            .all(|&weight| (weight - ((6.0_f64 / 4.0).ln() + 1.0)).abs() < 1e-12)
    );
}
