use serde::Serialize;
use uuid::Uuid;

use crate::{
    services::stream::models::{
        current_file::CurrentFile, explicit_context::ExplicitContext, model_details::ModelDetails,
    },
    GenerateInput,
};

use super::{
    code_chunk::CodeChunk,
    conversation::{Conversation, ConversationMessage, MessageType},
};

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RequestBody {
    pub current_file: CurrentFile,
    pub model_details: ModelDetails,

    #[serde(rename = "workspaceRootPath")]
    pub root_path: String,

    #[serde(rename = "explicitContext")]
    pub context: ExplicitContext,

    pub request_id: String,
    pub conversation: Conversation,
}

impl RequestBody {
    pub fn new_with_input(input: &GenerateInput) -> Self {
        let mut message = ConversationMessage::new(MessageType::User, input.prompt());
        message.attached_code_chunks.push(CodeChunk {
            relative_workspace_path: input.file_path(),
            start_line: input.selection_range().start().line(),
            lines: input
                .document_text()
                .lines()
                .map(|s| s.to_string())
                .collect(),
        });
        Self {
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
                api_key: input.api_key(),
            },
            root_path: input.workspace_directory().unwrap_or_default(),
            context: ExplicitContext {},
            request_id: Uuid::new_v4().to_string(),
            conversation: vec![message],
        }
    }
}
