#[derive(Serialize, Deserialize)]
pub struct Document {
    id: Uuid,
    title: String,
    page_count: usize,
    s3_url: String,
}

impl Document {
    pub fn new(id: Uuid, title: String, page_count: usize, s3_url: String) -> Self {
        Self {
            id,
            title,
            page_count,
            s3_url,
        }
    }

    pub fn id(&self) -> &Uuid {
        &self.id
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn page_count(&self) -> usize {
        self.page_count
    }

    pub fn url(&self) -> &str {
        &self.url
    }
}
