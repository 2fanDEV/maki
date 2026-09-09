use mongodb::{
    Collection, Database,
    bson::{Document, doc, oid::ObjectId},
    options::ReturnDocument,
};
use serde::{Deserialize, Serialize};

use crate::service::ServiceError;

pub mod api;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Label {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub name: String,
}

#[derive(Clone)]
pub struct LabelService {
    labels: Collection<Label>,
}

impl LabelService {
    pub fn new(database: &Database) -> Self {
        Self {
            labels: database.collection("labels"),
        }
    }

    pub async fn create(&self, name: &str) -> Result<Label, ServiceError> {
        let name = Self::name(name)?;
        // Omitting _id lets the MongoDB driver generate the ObjectId.
        let result = self
            .labels
            .clone_with_type::<Document>()
            .insert_one(doc! { "name": &name })
            .await?;
        let id = result.inserted_id.as_object_id().ok_or_else(|| {
            ServiceError::Internal(anyhow::anyhow!("MongoDB did not generate an ObjectId"))
        })?;
        Ok(Label { id, name })
    }

    pub async fn list(&self) -> Result<Vec<Label>, ServiceError> {
        let mut cursor = self.labels.find(doc! {}).sort(doc! { "_id": 1 }).await?;
        let mut labels = Vec::new();
        while cursor.advance().await? {
            labels.push(cursor.deserialize_current()?);
        }
        Ok(labels)
    }

    pub async fn get(&self, id: ObjectId) -> Result<Label, ServiceError> {
        self.labels
            .find_one(doc! { "_id": id })
            .await?
            .ok_or(ServiceError::NotFound("unknown label"))
    }

    pub async fn rename(&self, id: ObjectId, name: &str) -> Result<Label, ServiceError> {
        let name = Self::name(name)?;
        self.labels
            .find_one_and_update(doc! { "_id": id }, doc! { "$set": { "name": name } })
            .return_document(ReturnDocument::After)
            .await?
            .ok_or(ServiceError::NotFound("unknown label"))
    }

    pub async fn delete(&self, id: ObjectId) -> Result<(), ServiceError> {
        if self
            .labels
            .delete_one(doc! { "_id": id })
            .await?
            .deleted_count
            == 0
        {
            return Err(ServiceError::NotFound("unknown label"));
        }
        Ok(())
    }

    /// Used at trainer creation; deletion/reference protection is intentionally deferred.
    pub async fn validate_ids(&self, ids: &[ObjectId]) -> Result<(), ServiceError> {
        let mut ids = ids.to_vec();
        ids.sort_unstable();
        ids.dedup();
        let count = self
            .labels
            .count_documents(doc! { "_id": { "$in": &ids } })
            .await?;
        if count != ids.len() as u64 {
            return Err(ServiceError::InvalidInput(
                "one or more label IDs do not exist".into(),
            ));
        }
        Ok(())
    }

    fn name(name: &str) -> Result<String, ServiceError> {
        let name = name.trim();
        if name.is_empty() {
            return Err(ServiceError::InvalidInput(
                "label name must not be blank".into(),
            ));
        }
        Ok(name.to_owned())
    }
}
