use crate::document_service::s3util::determine_bucket;

pub mod api;
mod s3util;
pub mod types;

#[derive(Clone, Default)]
pub struct DocumentService {}

impl DocumentService {
    pub const DEFAULT_BUCKET: &str = "DEFAULT";
    pub fn new() -> Self {}

    fn upload_to_s3(
        &mut self,
        metadata: Map<String, Object>,
        bucket_name: Option<&str>,
        document: Vec<u8>,
    ) {
        let bucket = determine_bucket(bucket_name);
    }
}
