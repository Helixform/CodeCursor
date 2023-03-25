use serde::Serialize;

use crate::GenerateInput;

use super::{
    bot_message::BotMessage, code_area::CodeArea, split_code_into_blocks,
    user_message::UserMessage, UserRequest,
};

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MessageType {
    Edit,
    Generate,
    Freeform,
    Markdown,
}

#[derive(Debug, Serialize, Clone)]
pub struct RequestBody {
    #[serde(rename = "userRequest")]
    pub user_request: UserRequest,

    #[serde(rename = "botMessages")]
    pub bot_messages: Vec<BotMessage>,

    #[serde(rename = "userMessages")]
    pub user_messages: Vec<UserMessage>,

    #[serde(rename = "contextType")]
    pub context_type: String,

    #[serde(rename = "rootPath")]
    pub root_path: Option<String>,
}

impl RequestBody {
    pub fn new(
        user_request: UserRequest,
        user_messages: Vec<UserMessage>,
        bot_messages: Vec<BotMessage>,
        root_path: Option<String>,
    ) -> Self {
        Self {
            user_request,
            bot_messages,
            user_messages,
            context_type: "copilot".to_owned(),
            root_path,
        }
    }

    pub fn new_with_input(input: &GenerateInput, message_type: MessageType) -> Self {
        let file_path = input.file_path();
        let file_dir = input.file_dir();
        let area = CodeArea::new_with_input(input);
        let prompt = input.prompt();

        let user_request = UserRequest::new(
            prompt,
            file_dir,
            file_path.to_owned(),
            input.document_text(),
            split_code_into_blocks(&area.preceding_code),
            split_code_into_blocks(&area.following_code),
            area.selection_text,
            message_type,
        );

        let workspace_directory = input.workspace_directory();
        Self::new(user_request, vec![], vec![], workspace_directory)
    }
}
