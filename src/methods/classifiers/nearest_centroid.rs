use crate::methods::classifiers::Classifier;

struct NearestCentroid {}

impl Classifier for NearestCentroid {
    fn train(x: &sprs::CsMat<f64>, y: &[f64]) {
        todo!()
    }

    fn predict(&self, x: &[f64]) -> f64 {
        todo!()
    }
}

impl NearestCentroid {}
