mod nearest_centroid;

pub trait Classifier {
    fn train(x: &sprs::CsMat<f64>, y: &[f64]);
    fn predict(&self, x: &[f64]) -> f64;
}
