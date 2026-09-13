use serde::Serialize;

use crate::label_service::Label;

#[derive(Serialize, utoipa::ToSchema)]
pub(super) struct LabelResponse {
    pub id: String,
    pub name: String,
}

impl From<Label> for LabelResponse {
    fn from(label: Label) -> Self {
        Self {
            id: label.id.to_hex(),
            name: label.name,
        }
    }
}
