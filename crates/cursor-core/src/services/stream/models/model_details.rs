use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct ModelDetails {
    #[serde(rename = "modelName")]
    pub name: String,

    #[serde(rename = "enableGhostMode")]
    pub ghost_mode: bool,
}
