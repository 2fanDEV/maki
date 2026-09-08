pub mod api;
pub mod classification_service;
pub mod document_service;
mod macros;
pub mod methods;
pub mod service;

pub trait Model {
    fn classify(&self);
}
