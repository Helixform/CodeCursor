use serde::Serialize;

use crate::GenerateInput;

use super::{code_area::CodeArea, request_body::MessageType, split_code_into_blocks};

#[derive(Debug, Serialize, Clone)]
pub struct UserRequest {
    pub message: String,

    #[serde(rename = "currentRootPath")]
    pub current_root_path: String,

    #[serde(rename = "currentFileName")]
    pub current_file_name: String,

    #[serde(rename = "currentFileContents")]
    pub current_file_contents: String,

    #[serde(rename = "precedingCode")]
    pub preceding_code: Vec<String>,

    #[serde(rename = "suffixCode")]
    pub suffix_code: Vec<String>,

    #[serde(rename = "currentSelection")]
    pub current_selection: String,

    #[serde(rename = "copilotCodeBlocks")]
    pub copilot_code_blocks: Vec<String>,

    #[serde(rename = "customCodeBlocks")]
    pub custom_code_blocks: Vec<String>,

    #[serde(rename = "codeBlockIdentifiers")]
    pub code_block_identifiers: Vec<String>,

    #[serde(rename = "msgType")]
    pub message_type: MessageType,
}

impl UserRequest {
    pub fn new(
        message: String,
        current_root_path: String,
        current_file_name: String,
        current_file_contents: String,
        preceding_code: Vec<String>,
        suffix_code: Vec<String>,
        current_selection: String,
        message_type: MessageType,
    ) -> Self {
        Self {
            message,
            current_root_path,
            current_file_name,
            current_file_contents,
            preceding_code,
            suffix_code,
            current_selection,
            copilot_code_blocks: vec![],
            custom_code_blocks: vec![],
            code_block_identifiers: vec![],
            message_type,
        }
    }

    pub fn new_with_input(input: &GenerateInput, message_type: MessageType) -> Self {
        let file_path = input.file_path();
        let file_dir = input.file_dir();
        let code_area = CodeArea::new_with_input(input);
        let prompt = input.prompt();

        UserRequest::new(
            prompt,
            file_dir,
            file_path.to_owned(),
            input.document_text(),
            split_code_into_blocks(&code_area.preceding_code),
            split_code_into_blocks(&code_area.following_code),
            code_area.selection_text.unwrap_or_default(),
            message_type,
        )
    }
}
