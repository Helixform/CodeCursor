use serde::Serialize;

use crate::{
    services::stream::models::{
        current_file::CurrentFile, explicit_context::ExplicitContext, model_details::ModelDetails,
    },
    GenerateInput,
};

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RequestBody {
    pub query: String,
    pub current_file: CurrentFile,
    pub model_details: ModelDetails,

    #[serde(rename = "workspaceRootPath")]
    pub root_path: String,

    #[serde(rename = "apiKey", skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,

    #[serde(rename = "explicitContext")]
    pub context: ExplicitContext,
}

impl RequestBody {
    pub fn new_with_input(input: &GenerateInput) -> Self {
        Self {
            query: input.prompt(),
            current_file: CurrentFile {
                content: input.document_text(),
                language_id: input.language_id(),
                relative_workspace_path: input.file_path(),
                selection: input.selection_range().into(),
                cursor: input.cursor().into(),
            },
            model_details: ModelDetails {
                name: input.gpt_model(),
                ghost_mode: true,
            },
            root_path: input.workspace_directory().unwrap_or_default(),
            api_key: input.api_key(),
            context: ExplicitContext,
        }
    }
}
