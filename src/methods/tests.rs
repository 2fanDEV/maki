use super::*;
use classifiers::NearestCentroid;
use mongodb::bson::oid::ObjectId;
use preprocessing::{Document, tfidf::IdfWeightScheme};
use std::sync::{Arc, Mutex};

fn label(value: u8) -> ObjectId {
    ObjectId::from_bytes([value; 12])
}

#[derive(Clone)]
struct TestDocument {
    name: String,
    pages: Vec<String>,
    label: ObjectId,
}

impl Document for TestDocument {
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
impl LabeledDocument for TestDocument {
    fn label_id(&self) -> ObjectId {
        self.label
    }
}

fn documents() -> Vec<TestDocument> {
    [
        "ant", "bee", "cat", "dog", "elk", "fox", "gnu", "hen", "jay", "koi",
    ]
    .into_iter()
    .enumerate()
    .map(|(index, word)| TestDocument {
        name: word.to_owned(),
        pages: vec![format!(
            "{} {word}",
            if index < 5 { "apple" } else { "orange" }
        )],
        label: if index < 5 { label(3) } else { label(8) },
    })
    .collect()
}

fn trainer() -> ModelTrainer<TestDocument, NearestCentroid> {
    let mut builder = ModelTrainer::default();
    builder.documents(documents()).seed(42).method(Box::new(
        |docs: Vec<TestDocument>, features| {
            NearestCentroid::train(
                features,
                &docs.iter().map(|doc| doc.label_id()).collect::<Vec<_>>(),
            )
        },
    ));
    builder
}

#[test]
fn configuration_does_not_train_until_requested() {
    let calls = Arc::new(Mutex::new(0));
    let observed = Arc::clone(&calls);
    let mut trainer = trainer();
    trainer.method(Box::new(move |docs, features| {
        *observed.lock().unwrap() += 1;
        NearestCentroid::train(
            features,
            &docs
                .iter()
                .map(LabeledDocument::label_id)
                .collect::<Vec<_>>(),
        )
    }));
    assert_eq!(*calls.lock().unwrap(), 0);
    let outcome = trainer.train().unwrap();
    assert_eq!(*calls.lock().unwrap(), 1);
    assert_eq!(outcome.evaluation.selected_split, 0);
    assert_eq!(outcome.evaluation.seed, 42);
    assert_eq!(outcome.evaluation.splits[0].train_samples, 8);
    assert_eq!(outcome.evaluation.splits[0].test_samples, 2);
    assert_eq!(outcome.evaluation.splits[0].test.macro_f1, 1.0);
    assert_eq!(outcome.evaluation.score_usage, "held-out selection scores");
}

#[test]
fn repeated_splits_do_not_fit_test_terms_or_pass_test_labels_to_the_factory() {
    let passed_documents = Arc::new(Mutex::new(Vec::new()));
    let passed = Arc::clone(&passed_documents);
    let mut builder = trainer();
    builder
        .train_test_splits(SplitStrategy::REPEATED(3))
        .method(Box::new(move |docs, features| {
            passed.lock().unwrap().push(
                docs.iter()
                    .map(|doc| (doc.name.clone(), doc.label_id()))
                    .collect::<Vec<_>>(),
            );
            NearestCentroid::train(
                features,
                &docs.iter().map(|doc| doc.label_id()).collect::<Vec<_>>(),
            )
        }));
    let outcome = builder.train().unwrap();
    assert_eq!(passed_documents.lock().unwrap().len(), 3);
    let docs = documents();
    let splits = split::evaluated_splits(
        &docs,
        0.8,
        SplitStrategy::REPEATED(3),
        SamplingStrategy::Stratified,
        42,
    )
    .unwrap();
    for (index, split) in splits.iter().enumerate() {
        assert_eq!(
            passed_documents.lock().unwrap()[index],
            split
                .train_documents
                .iter()
                .map(|doc| (doc.name.clone(), doc.label_id()))
                .collect::<Vec<_>>()
        );
    }
    let selected = &splits[outcome.evaluation.selected_split];
    let vocabulary = outcome.tf_idf.vocabulary().unwrap();
    for doc in &selected.test_documents {
        assert!(!vocabulary.contains(&doc.name));
    }
    for doc in &selected.train_documents {
        assert!(vocabulary.contains(&doc.name));
    }
    let apple = vocabulary.iter().position(|word| word == "apple").unwrap();
    assert!((outcome.tf_idf.idf_weights().unwrap()[apple] - 2.0_f64.ln()).abs() < 1e-12);
}

#[test]
fn selected_model_and_tfidf_outlive_the_trainer() {
    let (outcome, training_documents) = {
        let mut builder = trainer();
        builder.train_test_splits(SplitStrategy::REPEATED(4));
        let outcome = builder.train().unwrap();
        let docs = documents();
        let splits = split::evaluated_splits(
            &docs,
            0.8,
            SplitStrategy::REPEATED(4),
            SamplingStrategy::Stratified,
            42,
        )
        .unwrap();
        let training = splits[outcome.evaluation.selected_split]
            .train_documents
            .iter()
            .map(|&doc| doc.clone())
            .collect::<Vec<_>>();
        (outcome, training)
    };
    let features = outcome.tf_idf.transform(&training_documents).unwrap();
    let labels: Vec<_> = training_documents
        .iter()
        .map(|doc| doc.label_id())
        .collect();
    assert_eq!(outcome.model.predict(&features), labels);
}

#[test]
fn custom_tfidf_settings_are_used_in_every_split() {
    let mut tf_idf = TfIdf::Builder();
    tf_idf.idf_weight_scheme(IdfWeightScheme::SMOOTH);
    let mut builder = trainer();
    builder
        .tf_idf_builder(tf_idf)
        .train_test_splits(SplitStrategy::REPEATED(2));
    let outcome = builder.train().unwrap();
    for report in &outcome.evaluation.splits {
        assert_eq!(report.test.macro_f1, 1.0);
    }
    {
        let tf_idf = &outcome.tf_idf;
        let apple = tf_idf
            .vocabulary()
            .unwrap()
            .iter()
            .position(|word| word == "apple")
            .unwrap();
        assert!(
            (tf_idf.idf_weights().unwrap()[apple] - ((8.0_f64 / 5.0).ln() + 1.0)).abs() < 1e-12
        );
    }
}

#[test]
fn invalid_configurations_fail_without_training() {
    let mut builder = trainer();
    for ratio in [0.0, 1.0, f32::NAN, -0.1, 1.1] {
        builder.split_ratio(ratio);
        assert!(builder.train().is_err());
    }
    builder
        .split_ratio(0.8)
        .train_test_splits(SplitStrategy::REPEATED(0));
    assert!(builder.train().is_err());
    builder
        .train_test_splits(SplitStrategy::ONCE)
        .documents(Vec::new());
    assert!(builder.train().is_err());
    let mut few = documents();
    few.truncate(1);
    builder.documents(few);
    assert!(builder.train().is_err());
    let mut tf_idf = TfIdf::Builder();
    tf_idf.double_normalization_k(f64::NAN);
    builder.documents(documents()).tf_idf_builder(tf_idf);
    assert!(builder.train().is_err());
}

#[test]
fn seeded_splits_preserve_classes_partition_input_and_repeat_exactly() {
    let documents = documents();
    for sampling in [SamplingStrategy::Stratified, SamplingStrategy::Random] {
        let first =
            split::evaluated_splits(&documents, 0.8, SplitStrategy::REPEATED(4), sampling, 123)
                .unwrap();
        let second =
            split::evaluated_splits(&documents, 0.8, SplitStrategy::REPEATED(4), sampling, 123)
                .unwrap();
        for (a, b) in first.iter().zip(second) {
            let names = |docs: &[&TestDocument]| {
                docs.iter().map(|doc| doc.name.clone()).collect::<Vec<_>>()
            };
            assert_eq!(names(&a.train_documents), names(&b.train_documents));
            assert_eq!(names(&a.test_documents), names(&b.test_documents));
            let mut all = names(&a.train_documents);
            all.extend(names(&a.test_documents));
            all.sort();
            all.dedup();
            assert_eq!(all.len(), documents.len());
            if sampling == SamplingStrategy::Stratified {
                for label in [label(3), label(8)] {
                    assert_eq!(
                        a.train_documents
                            .iter()
                            .filter(|doc| doc.label_id() == label)
                            .count(),
                        4
                    );
                    assert_eq!(
                        a.test_documents
                            .iter()
                            .filter(|doc| doc.label_id() == label)
                            .count(),
                        1
                    );
                }
            }
        }
    }
}

#[test]
fn small_stratified_classes_keep_both_partitions_at_extreme_valid_ratios() {
    let docs = documents();
    for ratio in [0.001, 0.999] {
        let splits = split::evaluated_splits(
            &docs,
            ratio,
            SplitStrategy::ONCE,
            SamplingStrategy::Stratified,
            1,
        )
        .unwrap();
        for label in [label(3), label(8)] {
            assert!(
                splits[0]
                    .train_documents
                    .iter()
                    .any(|doc| doc.label_id() == label)
            );
            assert!(
                splits[0]
                    .test_documents
                    .iter()
                    .any(|doc| doc.label_id() == label)
            );
        }
    }
}

#[test]
fn selecting_a_later_split_keeps_its_classifier_and_vocabulary_together() {
    struct Candidate {
        inner: NearestCentroid,
        inverted: bool,
    }
    impl Classifier for Candidate {
        fn train(x: &CsMat<f64>, y: &[ObjectId]) -> Result<Self> {
            Ok(Self {
                inner: NearestCentroid::train(x, y)?,
                inverted: false,
            })
        }
        fn labels(&self) -> Vec<ObjectId> {
            self.inner.labels()
        }
        fn predict(&self, x: &CsMat<f64>) -> Vec<ObjectId> {
            let mut labels = self.inner.predict(x);
            if self.inverted {
                for label in &mut labels {
                    *label = if *label == super::tests::label(3) {
                        super::tests::label(8)
                    } else {
                        super::tests::label(3)
                    };
                }
            }
            labels
        }
    }
    let outcome = {
        let mut calls = 0;
        let mut builder = ModelTrainer::<TestDocument, Candidate>::default();
        builder
            .documents(documents())
            .seed(42)
            .train_test_splits(SplitStrategy::REPEATED(2))
            .method(Box::new(move |docs, features| {
                let mut candidate = Candidate::train(
                    features,
                    &docs.iter().map(|doc| doc.label_id()).collect::<Vec<_>>(),
                )?;
                candidate.inverted = calls == 0;
                calls += 1;
                Ok(candidate)
            }));
        let outcome = builder.train().unwrap();
        assert_eq!(outcome.evaluation.selected_split, 1);
        let docs = documents();
        let splits = split::evaluated_splits(
            &docs,
            0.8,
            SplitStrategy::REPEATED(2),
            SamplingStrategy::Stratified,
            42,
        )
        .unwrap();
        let selected = &splits[1];
        let vocabulary = outcome.tf_idf.vocabulary().unwrap();
        for doc in &selected.train_documents {
            assert!(vocabulary.contains(&doc.name));
        }
        for doc in &selected.test_documents {
            assert!(!vocabulary.contains(&doc.name));
        }
        outcome
    };
    assert!(!outcome.model.inverted);
    assert_eq!(outcome.evaluation.splits[0].test.macro_f1, 0.0);
    assert_eq!(outcome.evaluation.splits[1].test.macro_f1, 1.0);
}

#[test]
fn evaluation_failure_fails_training_instead_of_skipping_a_candidate() {
    struct InvalidPredictions;
    impl Classifier for InvalidPredictions {
        fn train(_: &CsMat<f64>, _: &[ObjectId]) -> Result<Self> {
            Ok(Self)
        }
        fn predict(&self, _: &CsMat<f64>) -> Vec<ObjectId> {
            Vec::new()
        }
        fn labels(&self) -> Vec<ObjectId> {
            Vec::new()
        }
    }
    let mut trainer = ModelTrainer::<TestDocument, InvalidPredictions>::default();
    trainer
        .documents(documents())
        .method(Box::new(|_, _| Ok(InvalidPredictions)));
    assert!(
        trainer
            .train()
            .err()
            .unwrap()
            .to_string()
            .contains("metrics require")
    );
}

#[test]
fn missing_or_failing_factories_return_errors() {
    let mut trainer = ModelTrainer::<TestDocument, NearestCentroid>::default();
    trainer.documents(documents());
    assert_eq!(
        trainer.train().err().unwrap().to_string(),
        "Missing model generating function!"
    );
    let mut calls = 0;
    trainer
        .train_test_splits(SplitStrategy::REPEATED(3))
        .method(Box::new(move |docs, features| {
            calls += 1;
            if calls == 2 {
                return Err(anyhow!("second split failed"));
            }
            NearestCentroid::train(
                features,
                &docs
                    .iter()
                    .map(LabeledDocument::label_id)
                    .collect::<Vec<_>>(),
            )
        }));
    assert_eq!(
        trainer.train().err().unwrap().to_string(),
        "second split failed"
    );
}
