mod nearest_centroid;

pub use nearest_centroid::NearestCentroid;

pub trait Classifier {
    fn train(x: &sprs::CsMat<f64>, y: &[f64]) -> anyhow::Result<Self>
    where
        Self: Sized;

    /// Returns one label per document row, in the input matrix's row order.
    fn predict(&self, x: &sprs::CsMat<f64>) -> Vec<f64>;
}
