use serde::Serialize;

use super::{random, request_body::MessageType};

#[derive(Debug, Serialize, Clone)]
pub struct BotMessage {
    #[serde(rename = "sender")]
    pub sender: String,

    #[serde(rename = "sentAt")]
    pub sent_at: i64,

    #[serde(rename = "conversationId")]
    pub conversation_id: String,

    #[serde(rename = "type")]
    pub message_type: MessageType,

    #[serde(rename = "message")]
    pub message: String,

    #[serde(rename = "lastToken")]
    pub last_token: String,

    #[serde(rename = "finished")]
    pub finished: bool,

    #[serde(rename = "currentFile")]
    pub current_file: String,

    #[serde(rename = "interrupted")]
    pub interrupted: bool,

    #[serde(rename = "maxOrigLine")]
    pub max_original_line: i32,

    #[serde(rename = "hitTokenLimit")]
    pub hit_token_limit: bool,

    #[serde(rename = "useDiagnostics")]
    pub use_diagnostics: bool,
}

impl BotMessage {
    pub fn new(
        conversation_id: String,
        message_type: MessageType,
        message: String,
        last_token: String,
        current_file: String,
        finished: bool,
    ) -> Self {
        Self {
            sender: "bot".to_owned(),
            sent_at: chrono::Utc::now().timestamp_millis(),
            conversation_id,
            message_type,
            message,
            last_token,
            finished,
            current_file,
            interrupted: !finished,
            max_original_line: random(),
            hit_token_limit: true,
            use_diagnostics: false,
        }
    }
}
