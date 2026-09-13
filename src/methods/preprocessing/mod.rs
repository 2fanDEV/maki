use mongodb::bson::oid::ObjectId;
pub mod tfidf;
pub mod util;

pub trait Doc: Sync + Clone {
    fn name(&self) -> &str;
    fn pages(&self) -> &[String];
    fn pages_size(&self) -> i16;
}

/// A classification document referencing a label in MongoDB.
pub trait LabeledDocument: Doc {
    fn label_id(&self) -> ObjectId;
}
