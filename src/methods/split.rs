use anyhow::Result;
use rand::seq::SliceRandom;

use crate::invariant;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SplitStrategy {
    /// Shuffle and split once.
    ONCE,
    /// Independently shuffle and split the requested number of times.
    REPEATED(usize),
}

pub struct TrainTestSplit<'a, T> {
    pub train_documents: Vec<&'a T>,
    pub test_documents: Vec<&'a T>,
}

#[derive(
    utoipa::ToSchema,
    Clone,
    Copy,
    Debug,
    Default,
    Eq,
    PartialEq,
    serde::Deserialize,
    serde::Serialize,
)]
#[serde(rename_all = "snake_case")]
pub enum SamplingStrategy {
    Random,
    #[default]
    Stratified,
}

impl SplitStrategy {
    pub fn repetitions(self) -> usize {
        match self {
            Self::ONCE => 1,
            Self::REPEATED(count) => count,
        }
    }
}

/// Validate evaluated-training inputs without creating all requested splits.
pub(crate) fn validate_training_documents<T: super::LabeledDocument>(
    documents: &[T],
    ratio: f32,
    strategy: SplitStrategy,
    sampling: SamplingStrategy,
) -> Result<()> {
    invariant!(
        ratio.is_finite() && ratio > 0.0 && ratio < 1.0 && strategy.repetitions() > 0,
        "evaluated training requires a ratio between zero and one and at least one split"
    );
    invariant!(!documents.is_empty(), "training documents must be nonempty");
    let groups = class_groups(documents);
    invariant!(
        groups.iter().all(|group| group.len() >= 2),
        "every class needs at least two documents"
    );
    if sampling == SamplingStrategy::Random {
        let train_count = (documents.len() as f32 * ratio) as usize;
        invariant!(
            train_count > 0 && train_count < documents.len(),
            "training and test partitions must be nonempty"
        );
    }
    Ok(())
}

fn class_groups<T: super::LabeledDocument>(documents: &[T]) -> Vec<Vec<&T>> {
    let mut sorted: Vec<_> = documents.iter().collect();
    sorted.sort_by_key(|doc| doc.label_id());
    let mut groups: Vec<Vec<&T>> = Vec::new();
    for document in sorted {
        if let Some(group) = groups
            .last_mut()
            .filter(|group| group[0].label_id() == document.label_id())
        {
            group.push(document);
        } else {
            groups.push(vec![document]);
        }
    }
    groups
}

pub(super) fn evaluated_splits<T: super::LabeledDocument>(
    documents: &[T],
    ratio: f32,
    strategy: SplitStrategy,
    sampling: SamplingStrategy,
    seed: u64,
) -> Result<Vec<TrainTestSplit<'_, T>>> {
    use rand::SeedableRng;
    validate_training_documents(documents, ratio, strategy, sampling)?;
    let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
    if sampling == SamplingStrategy::Random {
        return random_splits(documents, ratio, strategy, &mut rng);
    }
    let groups = class_groups(documents);
    let mut splits = Vec::with_capacity(strategy.repetitions());
    for _ in 0..strategy.repetitions() {
        let mut train_documents = Vec::new();
        let mut test_documents = Vec::new();
        for group in &groups {
            let mut shuffled = group.clone();
            shuffled.shuffle(&mut rng);
            let train_count = ((group.len() as f32 * ratio) as usize).clamp(1, group.len() - 1);
            train_documents.extend_from_slice(&shuffled[..train_count]);
            test_documents.extend_from_slice(&shuffled[train_count..]);
        }
        train_documents.shuffle(&mut rng);
        test_documents.shuffle(&mut rng);
        splits.push(TrainTestSplit {
            train_documents,
            test_documents,
        });
    }
    Ok(splits)
}

/// Original random partition behavior: finite ratios clamp to [0, 1], sizes round down.
fn random_splits<'a, T>(
    documents: &'a [T],
    split_ratio: f32,
    strategy: SplitStrategy,
    rng: &mut impl rand::Rng,
) -> Result<Vec<TrainTestSplit<'a, T>>> {
    invariant!(split_ratio.is_finite(), "split ratio must be finite");
    invariant!(strategy.repetitions() > 0, "at least one split is required");
    let train_count = (documents.len() as f32 * split_ratio.clamp(0.0, 1.0)) as usize;
    let mut shuffled: Vec<_> = documents.iter().collect();
    let mut splits = Vec::with_capacity(strategy.repetitions());
    for _ in 0..strategy.repetitions() {
        shuffled.shuffle(rng);
        splits.push(TrainTestSplit {
            train_documents: shuffled[..train_count].to_vec(),
            test_documents: shuffled[train_count..].to_vec(),
        });
    }
    Ok(splits)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn train_test_splits<T>(
        documents: &[T],
        ratio: f32,
        strategy: SplitStrategy,
    ) -> Result<Vec<TrainTestSplit<'_, T>>> {
        random_splits(documents, ratio, strategy, &mut rand::rng())
    }

    #[test]
    fn repeated_splits_partition_every_document_without_changing_the_input() {
        // Deliberately does not implement Clone.
        struct Document(usize);
        let documents: Vec<_> = (0..10).map(Document).collect();
        let splits = train_test_splits(&documents, 0.7, SplitStrategy::REPEATED(5)).unwrap();

        assert_eq!(splits.len(), 5);
        for split in splits {
            assert_eq!(split.train_documents.len(), 7);
            assert_eq!(split.test_documents.len(), 3);
            let mut ids: Vec<_> = split
                .train_documents
                .iter()
                .chain(&split.test_documents)
                .map(|document| document.0)
                .collect();
            ids.sort_unstable();
            assert_eq!(ids, (0..10).collect::<Vec<_>>());
        }
        assert_eq!(
            documents
                .iter()
                .map(|document| document.0)
                .collect::<Vec<_>>(),
            (0..10).collect::<Vec<_>>()
        );
    }

    #[test]
    fn once_returns_one_complete_partition() {
        let documents = [0, 1, 2, 3, 4];
        let splits = train_test_splits(&documents, 0.8, SplitStrategy::ONCE).unwrap();

        assert_eq!(splits.len(), 1);
        assert_eq!(splits[0].train_documents.len(), 4);
        assert_eq!(splits[0].test_documents.len(), 1);
        let mut ids: Vec<_> = splits[0]
            .train_documents
            .iter()
            .chain(&splits[0].test_documents)
            .map(|&&document| document)
            .collect();
        ids.sort_unstable();
        assert_eq!(ids, documents);
    }

    #[test]
    fn rounds_training_size_down() {
        let splits = train_test_splits(&[0, 1, 2], 0.5, SplitStrategy::ONCE).unwrap();
        assert_eq!(splits[0].train_documents.len(), 1);
        assert_eq!(splits[0].test_documents.len(), 2);
    }

    #[test]
    fn clamps_finite_ratios_and_accepts_endpoints() {
        for strategy in [SplitStrategy::ONCE, SplitStrategy::REPEATED(5)] {
            for (ratio, train_count) in [
                (f32::MIN, 0),
                (-0.1, 0),
                (0.0, 0),
                (1.0, 2),
                (1.1, 2),
                (f32::MAX, 2),
            ] {
                let splits = train_test_splits(&[0, 1], ratio, strategy).unwrap();
                for split in splits {
                    assert_eq!(split.train_documents.len(), train_count);
                    assert_eq!(split.test_documents.len(), 2 - train_count);
                }
            }
        }
    }

    #[test]
    fn accepts_empty_partitions() {
        for documents in [&[][..], &[0][..], &[0, 1][..]] {
            let splits = train_test_splits(documents, 0.1, SplitStrategy::ONCE).unwrap();
            assert!(splits[0].train_documents.is_empty());
            assert_eq!(splits[0].test_documents.len(), documents.len());
        }
    }

    #[test]
    fn rejects_invalid_split_requests() {
        for strategy in [SplitStrategy::ONCE, SplitStrategy::REPEATED(5)] {
            for ratio in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
                assert!(train_test_splits(&[0, 1], ratio, strategy).is_err());
            }
        }
        assert!(train_test_splits(&[0, 1], 0.8, SplitStrategy::REPEATED(0)).is_err());
    }
}
