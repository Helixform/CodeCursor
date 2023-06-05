use serde::Serialize;

use super::code_chunk::CodeChunk;

pub type Conversation = Vec<ConversationMessage>;

#[derive(Debug, Clone)]
pub enum MessageType {
    User,
    Bot,
}

impl Serialize for MessageType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl ToString for MessageType {
    fn to_string(&self) -> String {
        match self {
            MessageType::User => "MESSAGE_TYPE_HUMAN".to_owned(),
            MessageType::Bot => "MESSAGE_TYPE_AI".to_owned(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConversationMessage {
    #[serde(rename = "type")]
    pub message_type: MessageType,

    #[serde(skip_serializing_if = "String::is_empty")]
    pub text: String,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub attached_code_chunks: Vec<CodeChunk>,
}

impl ConversationMessage {
    pub fn new(message_type: MessageType, text: String) -> Self {
        Self {
            message_type,
            text,
            attached_code_chunks: vec![],
        }
    }

    pub fn empty_message(msg_type: MessageType) -> Self {
        Self::new(msg_type, "".to_owned())
    }
}
