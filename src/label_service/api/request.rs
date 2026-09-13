use serde::Deserialize;

#[derive(Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub(super) struct LabelInput {
    pub name: String,
}

#[derive(Deserialize, schemars::JsonSchema)]
pub(super) struct LabelPath {
    pub id: String,
}
