use super::*;
use classifiers::NearestCentroid;
use preprocessing::{Document, tfidf::IdfWeightScheme};
use std::{cell::RefCell, rc::Rc};

#[derive(Clone)]
struct TestDocument {
    name: String,
    pages: Vec<String>,
    label: f64,
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
    fn label(&self) -> f64 {
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
        label: if index < 5 { -3.0 } else { 8.0 },
    })
    .collect()
}

fn builder() -> TrainerBuilder<TestDocument, NearestCentroid> {
    let mut builder = Trainer::Builder();
    builder.documents(documents()).seed(42).method(Box::new(
        |docs: Vec<TestDocument>, features| {
            NearestCentroid::train(
                features,
                &docs.iter().map(|doc| doc.label()).collect::<Vec<_>>(),
            )
        },
    ));
    builder
}

#[test]
fn default_builder_trains_and_evaluates_one_stratified_split() {
    let mut builder = builder();
    let trainer = builder.build().unwrap();
    assert_eq!(trainer.models().len(), 1);
    assert_eq!(trainer.tf_idfs().len(), 1);
    assert_eq!(trainer.train_test_documents()[0].train_documents.len(), 8);
    assert_eq!(trainer.train_test_documents()[0].test_documents.len(), 2);
    assert_eq!(trainer.evaluation().selected_split, 0);
    assert_eq!(trainer.evaluation().seed, 42);
    assert_eq!(trainer.evaluation().splits[0].test.macro_f1, 1.0);
    assert_eq!(
        trainer.evaluation().score_usage,
        "held-out selection scores"
    );
}

#[test]
fn repeated_splits_do_not_fit_test_terms_or_pass_test_labels_to_the_factory() {
    let passed_documents = Rc::new(RefCell::new(Vec::new()));
    let passed = Rc::clone(&passed_documents);
    let mut builder = builder();
    builder
        .train_test_splits(SplitStrategy::REPEATED(3))
        .method(Box::new(move |docs, features| {
            passed.borrow_mut().push(
                docs.iter()
                    .map(|doc| (doc.name.clone(), doc.label()))
                    .collect::<Vec<_>>(),
            );
            NearestCentroid::train(
                features,
                &docs.iter().map(|doc| doc.label()).collect::<Vec<_>>(),
            )
        }));
    let trainer = builder.build().unwrap();
    assert_eq!(passed_documents.borrow().len(), 3);
    for (index, split) in trainer.train_test_documents().iter().enumerate() {
        assert_eq!(
            passed_documents.borrow()[index],
            split
                .train_documents
                .iter()
                .map(|doc| (doc.name.clone(), doc.label()))
                .collect::<Vec<_>>()
        );
        let vocabulary = trainer.tf_idfs()[index].vocabulary().unwrap();
        for doc in &split.test_documents {
            assert!(!vocabulary.contains(&doc.name));
        }
        for doc in &split.train_documents {
            assert!(vocabulary.contains(&doc.name));
        }
        let weights = trainer.tf_idfs()[index].idf_weights().unwrap();
        let apple = vocabulary.iter().position(|word| word == "apple").unwrap();
        assert!((weights[apple] - 2.0_f64.ln()).abs() < 1e-12);
    }
}

#[test]
fn selected_model_and_tfidf_outlive_the_builder() {
    let (outcome, training_documents) = {
        let mut builder = builder();
        builder.train_test_splits(SplitStrategy::REPEATED(4));
        let trainer = builder.build().unwrap();
        let index = trainer.evaluation().selected_split;
        let docs: Vec<_> = trainer.train_test_documents()[index]
            .train_documents
            .iter()
            .map(|&doc| doc.clone())
            .collect();
        (trainer.into_best(), docs)
    };
    let features = outcome.tf_idf.transform(&training_documents).unwrap();
    let labels: Vec<_> = training_documents.iter().map(|doc| doc.label()).collect();
    assert_eq!(outcome.model.predict(&features), labels);
}

#[test]
fn custom_tfidf_settings_are_used_in_every_split() {
    let mut tf_idf = TfIdf::Builder();
    tf_idf.idf_weight_scheme(IdfWeightScheme::SMOOTH);
    let mut builder = builder();
    builder
        .tf_idf_builder(tf_idf)
        .train_test_splits(SplitStrategy::REPEATED(2));
    let trainer = builder.build().unwrap();
    for tf_idf in trainer.tf_idfs() {
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
fn trainer_reports_five_percent_per_successful_split_out_of_twenty() {
    let updates = Rc::new(RefCell::new(Vec::new()));
    let observed = Rc::clone(&updates);
    let mut builder = builder();
    builder
        .train_test_splits(SplitStrategy::REPEATED(20))
        .progress_observer(move |update| observed.borrow_mut().push(update));
    let trainer = builder.build().unwrap();
    let updates = updates.borrow();
    assert_eq!(updates.len(), 22); // Initial value, twenty trained classifiers, terminal state.
    for (index, update) in updates[..21].iter().enumerate() {
        assert_eq!(update.state, FittingState::Running);
        assert_eq!(update.total_splits, 20);
        assert_eq!(update.trained_splits, index);
        assert_eq!(update.percentage, index as f64 * 5.0);
    }
    assert_eq!(trainer.progress(), *updates.last().unwrap());
    assert_eq!(trainer.progress().state, FittingState::Completed);
    assert_eq!(trainer.progress().percentage, 100.0);
}

#[test]
fn progress_uses_the_actual_splits_received_by_the_trainer() {
    let documents = documents();
    let splits = split::evaluated_splits(
        &documents,
        0.8,
        SplitStrategy::REPEATED(3),
        SamplingStrategy::Stratified,
        42,
    )
    .unwrap();
    let mut updates = Vec::new();
    let mut creator: ModelCreatingFunction<TestDocument, NearestCentroid> =
        Box::new(|docs, features| {
            NearestCentroid::train(
                features,
                &docs.iter().map(|doc| doc.label()).collect::<Vec<_>>(),
            )
        });
    let trainer = Trainer::new(
        splits,
        &TfIdfBuilder::default(),
        &mut creator,
        &mut |update| updates.push(update),
        EvaluationStrategy::MacroF1,
        42,
    )
    .unwrap();
    assert_eq!(trainer.progress().total_splits, 3);
    assert!((updates[1].percentage - 100.0 / 3.0).abs() < 1e-12);
    assert!((updates[2].percentage - 200.0 / 3.0).abs() < 1e-12);
    assert_eq!(updates[3].percentage, 100.0);
}

#[test]
fn a_failed_fit_does_not_advance_the_percentage() {
    let updates = Rc::new(RefCell::new(Vec::new()));
    let observed = Rc::clone(&updates);
    let mut calls = 0;
    let mut builder = builder();
    builder
        .train_test_splits(SplitStrategy::REPEATED(5))
        .progress_observer(move |update| observed.borrow_mut().push(update))
        .method(Box::new(move |docs, features| {
            calls += 1;
            if calls == 2 {
                return Err(anyhow!("second split failed"));
            }
            NearestCentroid::train(
                features,
                &docs.iter().map(|doc| doc.label()).collect::<Vec<_>>(),
            )
        }));
    assert_eq!(
        builder.build().err().unwrap().to_string(),
        "second split failed"
    );
    let updates = updates.borrow();
    assert_eq!(
        updates
            .iter()
            .map(|update| update.percentage)
            .collect::<Vec<_>>(),
        vec![0.0, 20.0, 20.0]
    );
    assert_eq!(updates.last().unwrap().state, FittingState::Failed);
    assert_eq!(updates.last().unwrap().trained_splits, 1);
}

#[test]
fn missing_factory_keeps_its_default_error_and_reports_zero_percent() {
    let updates = Rc::new(RefCell::new(Vec::new()));
    let observed = Rc::clone(&updates);
    let mut missing = Trainer::<TestDocument, NearestCentroid>::Builder();
    missing
        .documents(documents())
        .progress_observer(move |update| observed.borrow_mut().push(update));
    assert_eq!(
        missing.build().err().unwrap().to_string(),
        "Missing model generating function!"
    );
    assert_eq!(updates.borrow().last().unwrap().state, FittingState::Failed);
    assert_eq!(updates.borrow().last().unwrap().percentage, 0.0);
}

#[test]
fn invalid_configurations_fail_without_training() {
    let mut builder = builder();
    for ratio in [0.0, 1.0, f32::NAN, -0.1, 1.1] {
        builder.split_ratio(ratio);
        assert!(builder.build().is_err());
    }
    builder
        .split_ratio(0.8)
        .train_test_splits(SplitStrategy::REPEATED(0));
    assert!(builder.build().is_err());
    builder
        .train_test_splits(SplitStrategy::ONCE)
        .documents(Vec::new());
    assert!(builder.build().is_err());
    let mut few = documents();
    few.truncate(1);
    builder.documents(few);
    assert!(builder.build().is_err());
    let mut invalid = documents();
    invalid[0].label = f64::NAN;
    builder.documents(invalid);
    assert!(builder.build().is_err());
    let mut tf_idf = TfIdf::Builder();
    tf_idf.double_normalization_k(f64::NAN);
    builder.documents(documents()).tf_idf_builder(tf_idf);
    assert!(builder.build().is_err());
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
                for label in [-3., 8.] {
                    assert_eq!(
                        a.train_documents
                            .iter()
                            .filter(|doc| doc.label() == label)
                            .count(),
                        4
                    );
                    assert_eq!(
                        a.test_documents
                            .iter()
                            .filter(|doc| doc.label() == label)
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
        for label in [-3., 8.] {
            assert!(
                splits[0]
                    .train_documents
                    .iter()
                    .any(|doc| doc.label() == label)
            );
            assert!(
                splits[0]
                    .test_documents
                    .iter()
                    .any(|doc| doc.label() == label)
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
        fn train(x: &CsMat<f64>, y: &[f64]) -> Result<Self> {
            Ok(Self {
                inner: NearestCentroid::train(x, y)?,
                inverted: false,
            })
        }
        fn predict(&self, x: &CsMat<f64>) -> Vec<f64> {
            let mut labels = self.inner.predict(x);
            if self.inverted {
                for label in &mut labels {
                    *label = if *label == -3.0 { 8.0 } else { -3.0 };
                }
            }
            labels
        }
    }
    let outcome = {
        let mut calls = 0;
        let mut builder = Trainer::<TestDocument, Candidate>::Builder();
        builder
            .documents(documents())
            .seed(42)
            .train_test_splits(SplitStrategy::REPEATED(2))
            .method(Box::new(move |docs, features| {
                let mut candidate = Candidate::train(
                    features,
                    &docs.iter().map(|doc| doc.label()).collect::<Vec<_>>(),
                )?;
                candidate.inverted = calls == 0;
                calls += 1;
                Ok(candidate)
            }));
        let trainer = builder.build().unwrap();
        assert_eq!(trainer.evaluation().selected_split, 1);
        let selected_vocabulary = trainer.tf_idfs()[1].vocabulary().unwrap().to_vec();
        let outcome = trainer.into_best();
        assert_eq!(outcome.tf_idf.vocabulary().unwrap(), selected_vocabulary);
        outcome
    };
    assert!(!outcome.model.inverted);
    assert_eq!(outcome.evaluation.splits[0].test.macro_f1, 0.0);
    assert_eq!(outcome.evaluation.splits[1].test.macro_f1, 1.0);
}

#[test]
fn evaluation_failure_fails_the_trainer_instead_of_skipping_a_candidate() {
    struct InvalidPredictions;
    impl Classifier for InvalidPredictions {
        fn train(_: &CsMat<f64>, _: &[f64]) -> Result<Self> {
            Ok(Self)
        }
        fn predict(&self, x: &CsMat<f64>) -> Vec<f64> {
            vec![f64::NAN; x.rows()]
        }
    }
    let updates = Rc::new(RefCell::new(Vec::new()));
    let observed = Rc::clone(&updates);
    let mut builder = Trainer::<TestDocument, InvalidPredictions>::Builder();
    builder
        .documents(documents())
        .progress_observer(move |update| observed.borrow_mut().push(update))
        .method(Box::new(|_, _| Ok(InvalidPredictions)));
    let error = builder.build().err().unwrap();
    assert!(error.to_string().contains("metrics require"));
    assert_eq!(updates.borrow().last().unwrap().state, FittingState::Failed);
    assert_eq!(updates.borrow().last().unwrap().percentage, 100.0); // Its classifier trained successfully; evaluation failed.
}
