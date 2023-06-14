use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelDetails {
    #[serde(rename = "modelName")]
    pub name: String,

    #[serde(rename = "enableGhostMode")]
    pub ghost_mode: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
}
