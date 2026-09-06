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

/// Each split samples independently; the same partition may occur again.
/// The training size is rounded down, with the remainder used for testing.
/// Finite ratios are clamped to [0, 1]; either partition may be empty.
pub(super) fn train_test_splits<T>(
    documents: &[T],
    split_ratio: f32,
    strategy: SplitStrategy,
) -> Result<Vec<TrainTestSplit<'_, T>>> {
    let repetitions = match strategy {
        SplitStrategy::ONCE => 1,
        SplitStrategy::REPEATED(repetitions) => repetitions,
    };
    invariant!(split_ratio.is_finite(), "split ratio must be finite");
    let split_ratio = split_ratio.clamp(0.0, 1.0);
    invariant!(repetitions > 0, "at least one split is required");
    let train_count = (documents.len() as f32 * split_ratio) as usize;

    let mut rng = rand::rng();
    let mut shuffled: Vec<_> = documents.iter().collect();
    let mut splits = Vec::with_capacity(repetitions);
    for _ in 0..repetitions {
        shuffled.shuffle(&mut rng);
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
