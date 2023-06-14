use serde::Serialize;

use crate::{
    context::get_extension_context,
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

    #[serde(rename = "explicitContext")]
    pub context: ExplicitContext,
}

impl RequestBody {
    pub fn new_with_input(input: &GenerateInput) -> Self {
        let context = get_extension_context();
        let configuration = context.model_configuration();
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
                name: configuration.model_name(),
                ghost_mode: true,
                api_key: configuration.api_key(),
            },
            root_path: input.workspace_directory().unwrap_or_default(),
            context: ExplicitContext {},
        }
    }
}
