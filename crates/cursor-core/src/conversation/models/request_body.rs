use serde::Serialize;

use crate::GenerateInput;

use super::{bot_message::BotMessage, user_message::UserMessage, UserRequest};

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
    pub root_path: String,

    #[serde(rename = "apiKey", skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,

    #[serde(rename = "customModel", skip_serializing_if = "Option::is_none")]
    pub gpt_model: Option<String>,
}

impl RequestBody {
    pub fn new(
        user_request: UserRequest,
        user_messages: Vec<UserMessage>,
        bot_messages: Vec<BotMessage>,
        root_path: Option<String>,
        api_key: Option<String>,
        gpt_model: Option<String>,
    ) -> Self {
        Self {
            user_request,
            bot_messages,
            user_messages,
            context_type: "copilot".to_owned(),
            root_path: root_path.unwrap_or_default(),
            api_key,
            gpt_model,
        }
    }

    pub fn new_with_input(input: &GenerateInput, message_type: MessageType) -> Self {
        Self::new(
            UserRequest::new_with_input(input, message_type),
            vec![],
            vec![],
            input.workspace_directory(),
            input.api_key(),
            input.gpt_model(),
        )
    }
}
