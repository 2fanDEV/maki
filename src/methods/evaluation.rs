use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::invariant;

#[derive(
    schemars::JsonSchema, Clone, Copy, Debug, Default, Eq, PartialEq, Deserialize, Serialize,
)]
#[serde(rename_all = "snake_case")]
pub enum EvaluationStrategy {
    #[default]
    MacroF1,
    BalancedAccuracy,
    MatthewsCorrelation,
}

#[derive(schemars::JsonSchema, Clone, Copy, Debug, Default, PartialEq, Serialize)]
pub struct Metrics {
    pub macro_f1: f64,
    pub balanced_accuracy: f64,
    pub matthews_correlation: f64,
}

impl Metrics {
    /// All metrics use the same confusion matrix, including binary macro-F1.
    pub fn calculate(expected: &[f64], predicted: &[f64]) -> Result<Self> {
        invariant!(
            !expected.is_empty()
                && expected.len() == predicted.len()
                && expected
                    .iter()
                    .chain(predicted)
                    .all(|value| value.is_finite()),
            "metrics require equally sized, nonempty, finite label vectors"
        );
        let mut labels: Vec<_> = expected.iter().chain(predicted).copied().collect();
        labels.sort_by(|a, b| a.partial_cmp(b).unwrap());
        labels.dedup();
        let index = |label: &f64| {
            labels
                .binary_search_by(|value| value.partial_cmp(label).unwrap())
                .unwrap()
        };
        let mut matrix = vec![vec![0usize; labels.len()]; labels.len()];
        for (actual, predicted) in expected.iter().zip(predicted) {
            matrix[index(actual)][index(predicted)] += 1;
        }
        let mut actual_counts = vec![0.0; labels.len()];
        let mut predicted_counts = vec![0.0; labels.len()];
        let mut correct = 0.0;
        for (actual, row) in matrix.iter().enumerate() {
            for (predicted, &count) in row.iter().enumerate() {
                actual_counts[actual] += count as f64;
                predicted_counts[predicted] += count as f64;
            }
            correct += row[actual] as f64;
        }
        let mut result = Self::default();
        let mut supported_classes = 0;
        for (class, row) in matrix.iter().enumerate() {
            let tp = row[class] as f64;
            let denominator = actual_counts[class] + predicted_counts[class];
            if denominator > 0.0 {
                result.macro_f1 += 2.0 * tp / denominator;
            }
            if actual_counts[class] > 0.0 {
                result.balanced_accuracy += tp / actual_counts[class];
                supported_classes += 1;
            }
        }
        result.macro_f1 /= labels.len() as f64;
        result.balanced_accuracy /= supported_classes as f64;
        let n = expected.len() as f64;
        let dot: f64 = actual_counts
            .iter()
            .zip(&predicted_counts)
            .map(|(a, p)| a * p)
            .sum();
        let actual_squares: f64 = actual_counts.iter().map(|v| v * v).sum();
        let predicted_squares: f64 = predicted_counts.iter().map(|v| v * v).sum();
        let denominator = ((n * n - actual_squares) * (n * n - predicted_squares)).sqrt();
        if denominator > 0.0 {
            result.matthews_correlation = ((correct * n - dot) / denominator).clamp(-1.0, 1.0);
        }
        Ok(result)
    }

    pub fn score(&self, strategy: EvaluationStrategy) -> f64 {
        match strategy {
            EvaluationStrategy::MacroF1 => self.macro_f1,
            EvaluationStrategy::BalancedAccuracy => self.balanced_accuracy,
            EvaluationStrategy::MatthewsCorrelation => self.matthews_correlation,
        }
    }

    fn zip(self, other: Self, f: impl Fn(f64, f64) -> f64) -> Self {
        Self {
            macro_f1: f(self.macro_f1, other.macro_f1),
            balanced_accuracy: f(self.balanced_accuracy, other.balanced_accuracy),
            matthews_correlation: f(self.matthews_correlation, other.matthews_correlation),
        }
    }
}

#[derive(schemars::JsonSchema, Clone, Debug, PartialEq, Serialize)]
pub struct SplitEvaluation {
    pub split_index: usize,
    pub train_samples: usize,
    pub test_samples: usize,
    pub train: Metrics,
    pub test: Metrics,
    pub gap: Metrics,
}

impl SplitEvaluation {
    pub(super) fn new(
        split_index: usize,
        train_samples: usize,
        test_samples: usize,
        train: Metrics,
        test: Metrics,
    ) -> Self {
        Self {
            split_index,
            train_samples,
            test_samples,
            train,
            test,
            gap: train.zip(test, |a, b| (a - b).abs()),
        }
    }
}

#[derive(schemars::JsonSchema, Clone, Debug, PartialEq, Serialize)]
pub struct MetricSummary {
    pub mean: Metrics,
    /// Population standard deviation across the observed splits.
    pub standard_deviation: Metrics,
}

impl MetricSummary {
    fn calculate(values: impl Iterator<Item = Metrics> + Clone, count: usize) -> Self {
        let mean = values.clone().fold(Metrics::default(), |a, b| {
            a.zip(b, |a, b| a + b / count as f64)
        });
        let variance = values.fold(Metrics::default(), |sum, value| {
            sum.zip(value.zip(mean, |a, b| (a - b).powi(2)), |a, b| {
                a + b / count as f64
            })
        });
        Self {
            mean,
            standard_deviation: variance.zip(variance, |a, _| a.sqrt()),
        }
    }
}

#[derive(schemars::JsonSchema, Clone, Debug, PartialEq, Serialize)]
pub struct EvaluationReport {
    pub strategy: EvaluationStrategy,
    pub seed: u64,
    pub score_usage: &'static str,
    pub selected_split: usize,
    pub splits: Vec<SplitEvaluation>,
    pub train_summary: MetricSummary,
    pub test_summary: MetricSummary,
    pub gap_summary: MetricSummary,
}

impl EvaluationReport {
    pub(super) fn new(
        strategy: EvaluationStrategy,
        seed: u64,
        splits: Vec<SplitEvaluation>,
    ) -> Self {
        // Descending test score; ascending gap and split index. Exact ties are deterministic.
        let selected_split = splits
            .iter()
            .min_by(|a, b| {
                b.test
                    .score(strategy)
                    .total_cmp(&a.test.score(strategy))
                    .then_with(|| a.gap.score(strategy).total_cmp(&b.gap.score(strategy)))
                    .then_with(|| a.split_index.cmp(&b.split_index))
            })
            .expect("evaluated trainers always have at least one split")
            .split_index;
        Self {
            strategy,
            seed,
            score_usage: "held-out selection scores",
            selected_split,
            train_summary: MetricSummary::calculate(splits.iter().map(|s| s.train), splits.len()),
            test_summary: MetricSummary::calculate(splits.iter().map(|s| s.test), splits.len()),
            gap_summary: MetricSummary::calculate(splits.iter().map(|s| s.gap), splits.len()),
            splits,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(actual: f64, expected: f64) {
        assert!((actual - expected).abs() < 1e-12, "{actual} != {expected}");
    }

    #[test]
    fn binary_macro_f1_includes_both_classes_and_handles_imbalance() {
        let metrics = Metrics::calculate(&[0., 0., 0., 1.], &[0., 0., 0., 0.]).unwrap();
        close(metrics.macro_f1, 3.0 / 7.0);
        close(metrics.balanced_accuracy, 0.5);
        close(metrics.matthews_correlation, 0.0);
    }

    #[test]
    fn multiclass_metrics_share_confusion_counts() {
        let metrics =
            Metrics::calculate(&[0., 0., 1., 1., 2., 2.], &[0., 1., 1., 1., 2., 0.]).unwrap();
        close(metrics.macro_f1, (0.5 + 0.8 + 2.0 / 3.0) / 3.0);
        close(metrics.balanced_accuracy, 2.0 / 3.0);
        close(
            metrics.matthews_correlation,
            12.0 / (22.0_f64 * 24.0).sqrt(),
        );
    }

    #[test]
    fn degenerate_and_wrong_predictions_have_finite_scores() {
        let metrics = Metrics::calculate(&[5., 5.], &[5., 5.]).unwrap();
        assert_eq!(
            metrics,
            Metrics {
                macro_f1: 1.,
                balanced_accuracy: 1.,
                matthews_correlation: 0.
            }
        );
        let wrong = Metrics::calculate(&[0., 1.], &[1., 0.]).unwrap();
        assert_eq!(
            wrong,
            Metrics {
                macro_f1: 0.,
                balanced_accuracy: 0.,
                matthews_correlation: -1.
            }
        );
        let zero = Metrics::calculate(&[-0., 0.], &[0., -0.]).unwrap();
        assert_eq!(zero.macro_f1, 1.);
        assert!(Metrics::calculate(&[], &[]).is_err());
        assert!(Metrics::calculate(&[0.], &[]).is_err());
        assert!(Metrics::calculate(&[f64::NAN], &[0.]).is_err());
    }

    #[test]
    fn selection_prefers_score_then_gap_then_index_for_each_strategy() {
        let metrics = |value| Metrics {
            macro_f1: value,
            balanced_accuracy: value,
            matthews_correlation: value,
        };
        for strategy in [
            EvaluationStrategy::MacroF1,
            EvaluationStrategy::BalancedAccuracy,
            EvaluationStrategy::MatthewsCorrelation,
        ] {
            let splits = vec![
                SplitEvaluation::new(0, 8, 2, metrics(0.2), metrics(0.2)),
                SplitEvaluation::new(1, 8, 2, metrics(1.0), metrics(0.8)),
                SplitEvaluation::new(2, 8, 2, metrics(0.9), metrics(0.8)),
                SplitEvaluation::new(3, 8, 2, metrics(0.9), metrics(0.8)),
            ];
            let report = EvaluationReport::new(strategy, 42, splits);
            assert_eq!(report.selected_split, 2);
            close(report.test_summary.mean.score(strategy), 0.65);
            close(
                report.test_summary.standard_deviation.score(strategy),
                0.0675_f64.sqrt(),
            );
        }
    }
}
