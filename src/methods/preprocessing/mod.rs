use mongodb::bson::oid::ObjectId;
pub mod tfidf;
pub mod util;

pub trait Document: Sync + Clone {
    fn name(&self) -> &str;
    fn pages(&self) -> &[String];
    fn pages_size(&self) -> i16;
}

/// A classification document referencing a label in MongoDB.
pub trait LabeledDocument: Document {
    fn label_id(&self) -> ObjectId;
}
