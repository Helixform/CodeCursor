use serde::Serialize;

use crate::GenerateInput;

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MessageType {
    Edit,
    Generate,
    Freeform,
}

#[derive(Debug, Clone, Serialize)]
pub struct ModelDetails {
    #[serde(rename = "modelName")]
    pub name: String,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RequestBody {
    pub query: String,
    pub model_details: ModelDetails,

    #[serde(rename = "workspaceRootPath")]
    pub root_path: String,

    #[serde(rename = "apiKey", skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
}

impl RequestBody {
    pub fn new_with_input(input: &GenerateInput, message_type: MessageType) -> Self {
        Self {
            query: input.prompt(),
            model_details: ModelDetails {
                name: input.gpt_model().unwrap(),
            },
            root_path: input.workspace_directory().unwrap_or_default(),
            api_key: input.api_key(),
        }
    }
}
