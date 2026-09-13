use serde::Deserialize;

#[derive(Deserialize, utoipa::ToSchema)]
#[serde(deny_unknown_fields)]
pub(super) struct LabelInput {
    pub name: String,
}

#[derive(Deserialize, utoipa::ToSchema)]
pub(super) struct LabelPath {
    pub id: String,
}
