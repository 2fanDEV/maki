mod nearest_centroid;
use mongodb::bson::oid::ObjectId;

pub use nearest_centroid::NearestCentroid;

pub trait Classifier {
    fn train(x: &sprs::CsMat<f64>, y: &[ObjectId]) -> anyhow::Result<Self>
    where
        Self: Sized;

    /// Returns one label per document row, in the input matrix's row order.
    fn predict(&self, x: &sprs::CsMat<f64>) -> Vec<ObjectId>;

    /// Distinct learned label IDs in sorted order.
    fn labels(&self) -> Vec<ObjectId>;
}
