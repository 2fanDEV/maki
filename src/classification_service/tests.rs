use super::*;
use crate::service::Service;
use axum::{
    body::Body,
    http::{Request, StatusCode},
    response::Response,
};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use tokio::time::{Duration, timeout};
use tower::ServiceExt;

fn input() -> Value {
    json!({
        "classifier": "nearest_centroid",
        "documents": (0..10).map(|index| json!({
            "name": format!("doc-{index}"),
            "pages": [if index < 5 { "apple fruit sweet" } else { "orange citrus sour" }],
            "label": if index < 5 { -3.0 } else { 8.0 },
        })).collect::<Vec<_>>(),
        "seed": 42,
    })
}
async fn send(service: &ClassificationService, method: &str, uri: &str, body: String) -> Response {
    service
        .clone()
        .router()
        .oneshot(
            Request::builder()
                .method(method)
                .uri(uri)
                .header("content-type", "application/json")
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap()
}
async fn json_response(response: Response) -> Value {
    serde_json::from_slice(&response.into_body().collect().await.unwrap().to_bytes()).unwrap()
}
async fn submit(service: &ClassificationService, value: Value) -> Value {
    let response = timeout(
        Duration::from_secs(5),
        send(service, "POST", "/models", value.to_string()),
    )
    .await
    .unwrap();
    let status = response.status();
    let body = json_response(response).await;
    assert_eq!(status, StatusCode::ACCEPTED, "{body}");
    body
}
async fn finished(service: &ClassificationService, id: Uuid) -> ModelMetadata {
    let mut receiver = service.receiver(id).await.unwrap();
    timeout(Duration::from_secs(5), async {
        loop {
            let metadata = receiver.borrow_and_update().clone();
            if matches!(
                metadata.progress.state,
                FittingState::Completed | FittingState::Failed
            ) {
                return metadata;
            }
            receiver.changed().await.unwrap();
        }
    })
    .await
    .unwrap()
}
fn sse_updates(bytes: &[u8]) -> Vec<Value> {
    std::str::from_utf8(bytes)
        .unwrap()
        .lines()
        .filter_map(|line| line.strip_prefix("data:"))
        .map(|data| serde_json::from_str(data.trim()).unwrap())
        .collect()
}

#[tokio::test]
async fn model_creation_publishes_a_usable_winner_under_the_original_model_id() {
    let service = ClassificationService::default();
    let accepted = submit(&service, input()).await;
    let id: Uuid = serde_json::from_value(accepted["model_id"].clone()).unwrap();
    assert_eq!(accepted["seed"], 42);
    assert_eq!(accepted["status_url"], format!("/models/{id}"));
    assert_eq!(accepted["events_url"], format!("/models/{id}/events"));
    assert_eq!(accepted.as_object().unwrap().len(), 4);
    let completed = finished(&service, id).await;
    assert_eq!(completed.model_id, id);
    assert_eq!(completed.progress.state, FittingState::Completed);
    assert_eq!(completed.progress.percentage, 100.0);
    assert_eq!(completed.progress.trained_splits, 5);
    assert_eq!(completed.progress.total_splits, 5);
    assert_eq!(completed.classifier, ClassifierType::NearestCentroid);
    assert_eq!(completed.evaluation.as_ref().unwrap().splits.len(), 5);
    let polled =
        json_response(send(&service, "GET", &format!("/models/{id}"), String::new()).await).await;
    let expected: Value = serde_json::from_slice(&serde_json::to_vec(&completed).unwrap()).unwrap();
    assert_eq!(polled, expected);
    let outcome = service.model(id).await.unwrap();
    assert_eq!(outcome.progress, completed.progress);
    assert_eq!(&outcome.evaluation, completed.evaluation.as_ref().unwrap());
    let documents = [
        TrainingDocument {
            name: "new apple".into(),
            pages: vec!["apple sweet".into()],
            label: -3.0,
        },
        TrainingDocument {
            name: "new orange".into(),
            pages: vec!["orange".into()],
            label: 8.0,
        },
    ];
    let features = outcome.tf_idf.transform(&documents).unwrap();
    assert_eq!(outcome.model.predict(&features), [-3.0, 8.0]);
}

#[test]
fn creation_returns_before_training_and_sse_reports_model_percentages() {
    // Hold the executor's sole blocking thread to observe the creation response before fitting starts.
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .max_blocking_threads(1)
        .build()
        .unwrap();
    let (release, blocked) = std::sync::mpsc::channel();
    let blocker = runtime.spawn_blocking(move || blocked.recv().unwrap());
    runtime.block_on(async {
        let service = ClassificationService::default();
        let accepted = submit(&service, input()).await;
        let id = serde_json::from_value(accepted["model_id"].clone()).unwrap();
        assert!(service.model(id).await.is_none());
        let response = send(
            &service,
            "GET",
            accepted["events_url"].as_str().unwrap(),
            String::new(),
        )
        .await;
        assert_eq!(response.headers()["content-type"], "text/event-stream");
        let mut body = response.into_body();
        let initial = timeout(Duration::from_secs(5), body.frame())
            .await
            .unwrap()
            .unwrap()
            .unwrap()
            .into_data()
            .unwrap();
        let initial = sse_updates(&initial);
        assert_eq!(initial[0]["progress"]["state"], "initialized");
        assert_eq!(initial[0]["progress"]["percentage"], 0.0);
        assert_eq!(initial[0]["progress"]["total_splits"], 0); // The trainer has not received its splits yet.
        release.send(()).unwrap();
        let bytes = timeout(Duration::from_secs(5), body.collect())
            .await
            .unwrap()
            .unwrap()
            .to_bytes();
        let updates = sse_updates(&bytes);
        assert_eq!(updates.last().unwrap()["progress"]["state"], "completed");
        assert_eq!(updates.last().unwrap()["progress"]["percentage"], 100.0);
        for update in &updates {
            let progress = &update["progress"];
            assert_eq!(progress["total_splits"], 5);
            assert_eq!(
                progress["percentage"].as_f64().unwrap(),
                progress["trained_splits"].as_u64().unwrap() as f64 * 20.0
            );
        }
        assert!(
            updates
                .windows(2)
                .all(|pair| pair[0]["progress"]["percentage"].as_f64()
                    <= pair[1]["progress"]["percentage"].as_f64())
        );
        let polled = json_response(
            send(
                &service,
                "GET",
                accepted["status_url"].as_str().unwrap(),
                String::new(),
            )
            .await,
        )
        .await;
        assert_eq!(updates.last().unwrap(), &polled);
        let response = send(
            &service,
            "GET",
            accepted["events_url"].as_str().unwrap(),
            String::new(),
        )
        .await;
        let bytes = timeout(Duration::from_secs(5), response.into_body().collect())
            .await
            .unwrap()
            .unwrap()
            .to_bytes();
        assert_eq!(sse_updates(&bytes), vec![polled]);
    });
    runtime.block_on(blocker).unwrap();
}

#[tokio::test]
async fn disconnecting_sse_does_not_cancel_model_creation() {
    let service = ClassificationService::default();
    let accepted = submit(&service, input()).await;
    drop(
        send(
            &service,
            "GET",
            accepted["events_url"].as_str().unwrap(),
            String::new(),
        )
        .await,
    );
    let id = serde_json::from_value(accepted["model_id"].clone()).unwrap();
    assert_eq!(
        finished(&service, id).await.progress.state,
        FittingState::Completed
    );
}

#[tokio::test]
async fn failed_training_preserves_percentage_and_exposes_no_fitted_model() {
    let service = ClassificationService::default();
    let mut request = input();
    for document in request["documents"].as_array_mut().unwrap() {
        document["pages"] = json!([""]);
    }
    let accepted = submit(&service, request).await;
    let id = serde_json::from_value(accepted["model_id"].clone()).unwrap();
    let metadata = finished(&service, id).await;
    assert_eq!(metadata.progress.state, FittingState::Failed);
    assert_eq!(metadata.progress.percentage, 0.0);
    assert_eq!(metadata.progress.trained_splits, 0);
    assert_eq!(metadata.progress.total_splits, 5);
    assert!(metadata.error.unwrap().contains("training data"));
    assert!(metadata.evaluation.is_none());
    assert!(service.model(id).await.is_none());
    let response = send(
        &service,
        "GET",
        accepted["events_url"].as_str().unwrap(),
        String::new(),
    )
    .await;
    let bytes = timeout(Duration::from_secs(5), response.into_body().collect())
        .await
        .unwrap()
        .unwrap()
        .to_bytes();
    assert_eq!(
        sse_updates(&bytes).last().unwrap()["progress"]["state"],
        "failed"
    );
}

#[tokio::test]
async fn classifier_is_required_and_unsupported_types_are_rejected() {
    let service = ClassificationService::default();
    let mut request = input();
    request.as_object_mut().unwrap().remove("classifier");
    assert_eq!(
        send(&service, "POST", "/models", request.to_string())
            .await
            .status(),
        StatusCode::BAD_REQUEST
    );
    request["classifier"] = json!("svm");
    assert_eq!(
        send(&service, "POST", "/models", request.to_string())
            .await
            .status(),
        StatusCode::BAD_REQUEST
    );
    assert!(service.models.read().await.is_empty());
}

#[tokio::test]
async fn invalid_requests_and_unknown_models_have_defined_http_statuses() {
    let service = ClassificationService::default();
    for body in ["{", "{}"] {
        assert_eq!(
            send(&service, "POST", "/models", body.into())
                .await
                .status(),
            StatusCode::BAD_REQUEST
        );
    }
    for patch in [
        json!({"ratio": 0.0}),
        json!({"ratio": 1.0}),
        json!({"repetitions": 0}),
    ] {
        let mut request = input();
        request["split"] = patch;
        assert_eq!(
            send(&service, "POST", "/models", request.to_string())
                .await
                .status(),
            StatusCode::UNPROCESSABLE_ENTITY
        );
    }
    let mut request = input();
    request["documents"] = json!([]);
    assert_eq!(
        send(&service, "POST", "/models", request.to_string())
            .await
            .status(),
        StatusCode::UNPROCESSABLE_ENTITY
    );
    let mut request = input();
    request["tf_idf"] = json!({"double_normalization_k": 2.0});
    assert_eq!(
        send(&service, "POST", "/models", request.to_string())
            .await
            .status(),
        StatusCode::UNPROCESSABLE_ENTITY
    );
    for uri in [
        format!("/models/{}", Uuid::new_v4()),
        format!("/models/{}/events", Uuid::new_v4()),
    ] {
        assert_eq!(
            send(&service, "GET", &uri, String::new()).await.status(),
            StatusCode::NOT_FOUND
        );
    }
    assert!(service.models.read().await.is_empty());
}

#[tokio::test]
async fn metric_seed_and_tfidf_settings_are_preserved() {
    let service = ClassificationService::default();
    let mut request = input();
    request["evaluation_strategy"] = json!("matthews_correlation");
    request["split"] = json!({"ratio": 0.6, "repetitions": 2});
    request["tf_idf"] = json!({"idf_scheme": "smooth", "tf_scheme": "binary"});
    let a = submit(&service, request.clone()).await;
    let b = submit(&service, request).await;
    let a = finished(
        &service,
        serde_json::from_value(a["model_id"].clone()).unwrap(),
    )
    .await;
    let b = finished(
        &service,
        serde_json::from_value(b["model_id"].clone()).unwrap(),
    )
    .await;
    assert_eq!(a.evaluation, b.evaluation);
    let report = a.evaluation.unwrap();
    assert_eq!(
        report.strategy,
        crate::methods::EvaluationStrategy::MatthewsCorrelation
    );
    assert_eq!(report.splits.len(), 2);
    assert_eq!(report.splits[0].train_samples, 6);
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

#[tokio::test]
async fn generated_seed_is_returned_and_can_reproduce_the_report() {
    let service = ClassificationService::default();
    let mut request = input();
    request.as_object_mut().unwrap().remove("seed");
    let first = submit(&service, request.clone()).await;
    let first_report = finished(
        &service,
        serde_json::from_value(first["model_id"].clone()).unwrap(),
    )
    .await
    .evaluation
    .unwrap();
    assert_eq!(first_report.seed, first["seed"].as_u64().unwrap());
    request["seed"] = first["seed"].clone();
    let second = submit(&service, request).await;
    let second_report = finished(
        &service,
        serde_json::from_value(second["model_id"].clone()).unwrap(),
    )
    .await
    .evaluation
    .unwrap();
    assert_eq!(first_report, second_report);
}
