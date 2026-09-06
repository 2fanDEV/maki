mod classification_service;
mod document_service;
pub mod preprocessing;
mod service;

pub trait Model {
    fn classify(&self);
}
