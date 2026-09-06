mod classification_service;
mod document_service;
mod methods;
mod service;

pub trait Model {
    fn classify(&self);
}
