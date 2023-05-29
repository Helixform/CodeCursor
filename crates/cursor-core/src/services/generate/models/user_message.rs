use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct UserMessage {
    #[serde(rename = "sender")]
    pub sender: String,

    #[serde(rename = "sentAt")]
    pub index: usize,

    #[serde(rename = "message")]
    pub message: String,
}

impl UserMessage {
    pub fn new(message: String, index: usize) -> Self {
        Self {
            sender: "user".to_owned(),
            index,
            message,
        }
    }
}
