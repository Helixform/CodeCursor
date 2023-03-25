use serde::Serialize;

use crate::{models::code_area::CodeArea, GenerateInput};

use super::MessageType;

#[derive(Debug, Clone, Serialize)]
pub struct Selection {
    from: usize,
    to: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct UserMessage {
    #[serde(rename = "sender")]
    pub sender: String,

    #[serde(rename = "sentAt")]
    pub sent_at: i64,

    #[serde(rename = "message")]
    pub message: String,

    #[serde(rename = "conversationId")]
    pub conversation_id: String,

    #[serde(rename = "otherCodeBlocks")]
    pub other_code_blocks: Vec<String>,

    #[serde(rename = "codeSymbols")]
    pub code_symbols: Vec<String>,

    #[serde(rename = "currentFile")]
    pub current_file: String,

    #[serde(rename = "precedingCode")]
    pub preceding_code: String,

    #[serde(rename = "procedingCode")]
    pub proceeding_code: String,

    #[serde(rename = "currentSelection")]
    pub current_selection: Option<String>,

    #[serde(rename = "msgType")]
    pub message_type: MessageType,

    #[serde(rename = "selection")]
    pub selection: Selection,
}

impl UserMessage {
    pub fn new(
        message: String,
        conversation_id: String,
        current_file: String,
        preceding_code: String,
        proceeding_code: String,
        current_selection: Option<String>,
        message_type: MessageType,
        selection: Selection,
    ) -> Self {
        Self {
            sender: "user".to_owned(),
            sent_at: chrono::Utc::now().timestamp_millis(),
            message,
            conversation_id,
            other_code_blocks: vec![],
            code_symbols: vec![],
            current_file,
            preceding_code,
            proceeding_code,
            current_selection,
            message_type,
            selection,
        }
    }

    pub fn new_with_input(
        input: &GenerateInput,
        conversation_id: &str,
        message_type: MessageType,
    ) -> Self {
        let code_area = CodeArea::new_with_input(input);
        let range = input.selection_range();
        let selection = Selection {
            from: range.offset(),
            to: range.offset() + range.length(),
        };
        Self::new(
            input.prompt(),
            conversation_id.to_owned(),
            input.file_path(),
            code_area.preceding_code,
            code_area.following_code,
            code_area.selection_text,
            message_type,
            selection,
        )
    }
}
