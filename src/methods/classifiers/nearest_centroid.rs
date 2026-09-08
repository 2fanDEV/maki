use anyhow::Result;
use sprs::CsMat;

use crate::{invariant, methods::classifiers::Classifier};

#[derive(Debug)]
pub struct NearestCentroid {
    labels: Vec<f64>,
    centroids: Vec<Vec<f64>>,
}

impl Classifier for NearestCentroid {
    fn train(x: &CsMat<f64>, y: &[f64]) -> Result<Self> {
        invariant!(
            x.rows() > 0 && x.cols() > 0 && x.rows() == y.len(),
            "training data must have samples, features, and one label per sample"
        );
        invariant!(
            y.iter().chain(x.data()).all(|value| value.is_finite()),
            "training features and labels must be finite"
        );

        let mut labels = y.to_vec();
        labels.sort_by(|a, b| a.partial_cmp(b).unwrap());
        labels.dedup();
        let mut centroids = vec![vec![0.0; x.cols()]; labels.len()];
        let mut counts = vec![0usize; labels.len()];
        let classes: Vec<_> = y
            .iter()
            .map(|label| {
                labels
                    .binary_search_by(|value| value.partial_cmp(label).unwrap())
                    .unwrap()
            })
            .collect();

        for &class in &classes {
            counts[class] += 1;
        }

        let rows = x.to_csr();
        for (row, &class) in rows.outer_iterator().zip(&classes) {
            for (column, value) in row.iter() {
                centroids[class][column] += value / counts[class] as f64;
            }
        }

        Ok(Self { labels, centroids })
    }

    fn predict(&self, x: &CsMat<f64>) -> Vec<f64> {
        assert_eq!(
            x.cols(),
            self.centroids[0].len(),
            "sample feature count must match training data"
        );
        assert!(
            x.data().iter().all(|value| value.is_finite()),
            "features must be finite"
        );

        let converted;
        let rows = if x.is_csr() {
            x.view()
        } else {
            converted = x.to_csr();
            converted.view()
        };
        rows.outer_iterator()
            .map(|row| {
                let mut closest = 0;
                let mut shortest_distance = f64::INFINITY;
                for (class, centroid) in self.centroids.iter().enumerate() {
                    let mut entries = row.iter().peekable();
                    let distance =
                        centroid
                            .iter()
                            .enumerate()
                            .fold(0.0_f64, |distance, (column, mean)| {
                                // Missing sparse entries contribute zero, including all-zero rows.
                                let value = entries
                                    .next_if(|(index, _)| *index == column)
                                    .map_or(0.0, |(_, value)| *value);
                                distance.hypot(value - mean)
                            });
                    if distance < shortest_distance {
                        closest = class;
                        shortest_distance = distance;
                    }
                }
                self.labels[closest]
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    fn training_data() -> CsMat<f64> {
        // Means are [1, 2] for label -3, and [7, 0] for label 8.
        CsMat::new(
            (4, 2),
            vec![0, 1, 3, 4, 5],
            vec![1, 0, 1, 0, 0],
            vec![2.0, 2.0, 2.0, 6.0, 8.0],
        )
    }

    #[rstest]
    #[case(false)]
    #[case(true)]
    fn learns_class_means_and_predicts_with_either_sparse_layout(#[case] csc: bool) {
        let matrix = training_data();
        let matrix = if csc { matrix.to_csc() } else { matrix };
        let model = NearestCentroid::train(&matrix, &[-3.0, -3.0, 8.0, 8.0]).unwrap();

        assert_eq!(model.centroids, [vec![1.0, 2.0], vec![7.0, 0.0]]);
        // Includes a tie, missing features, and an entirely empty row.
        let features = CsMat::new(
            (5, 2),
            vec![0, 2, 4, 6, 7, 7],
            vec![0, 1, 0, 1, 0, 1, 0],
            vec![1.0, 1.0, 6.0, 1.0, 4.0, 1.0, 7.0],
        );
        let features = if csc { features.to_csc() } else { features };
        assert_eq!(model.predict(&features), [-3.0, 8.0, -3.0, 8.0, -3.0]);
        assert!(model.predict(&CsMat::zero((0, 2))).is_empty());
    }

    #[test]
    fn averages_implicit_zeros_and_accepts_a_single_class() {
        let matrix = CsMat::new((3, 1), vec![0, 1, 1, 1], vec![0], vec![6.0]);
        let model = NearestCentroid::train(&matrix, &[5.0; 3]).unwrap();
        assert_eq!(model.centroids, [vec![2.0]]);
        let features = CsMat::new((1, 1), vec![0, 1], vec![0], vec![100.0]);
        assert_eq!(model.predict(&features), [5.0]);
    }

    #[test]
    fn rejects_invalid_training_data() {
        assert!(NearestCentroid::train(&CsMat::zero((0, 2)), &[]).is_err());
        assert!(NearestCentroid::train(&CsMat::zero((1, 0)), &[0.0]).is_err());
        assert!(NearestCentroid::train(&training_data(), &[1.0]).is_err());
        for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert!(NearestCentroid::train(&training_data(), &[value; 4]).is_err());
            let matrix = CsMat::new((1, 1), vec![0, 1], vec![0], vec![value]);
            assert!(NearestCentroid::train(&matrix, &[1.0]).is_err());
        }
    }

    #[test]
    #[should_panic(expected = "sample feature count must match training data")]
    fn rejects_wrong_prediction_dimensions() {
        let model = NearestCentroid::train(&training_data(), &[1.0; 4]).unwrap();
        model.predict(&CsMat::zero((1, 1)));
    }

    #[test]
    #[should_panic(expected = "features must be finite")]
    fn rejects_non_finite_predictions() {
        let model = NearestCentroid::train(&training_data(), &[1.0; 4]).unwrap();
        let features = CsMat::new((1, 2), vec![0, 1], vec![0], vec![f64::NAN]);
        model.predict(&features);
    }
}
