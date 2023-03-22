use serde::Serialize;

use super::{bot_message::BotMessage, UserRequest};

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MessageType {
    Edit,
    Generate,
}

#[derive(Debug, Serialize, Clone)]
pub struct RequestBody {
    #[serde(rename = "userRequest")]
    pub user_request: UserRequest,

    #[serde(rename = "botMessages")]
    pub bot_messages: Vec<BotMessage>,

    #[serde(rename = "userMessages")]
    pub user_messages: Vec<String>,

    #[serde(rename = "contextType")]
    pub context_type: String,

    #[serde(rename = "rootPath")]
    pub root_path: Option<String>,
}

impl RequestBody {
    pub fn new(
        user_request: UserRequest,
        bot_messages: Vec<BotMessage>,
        root_path: Option<String>,
    ) -> Self {
        Self {
            user_request,
            bot_messages,
            user_messages: vec![],
            context_type: "copilot".to_owned(),
            root_path,
        }
    }
}
