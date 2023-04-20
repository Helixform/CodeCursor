use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct BotMessage {
    #[serde(rename = "sender")]
    pub sender: String,

    #[serde(rename = "sentAt")]
    pub index: usize,

    #[serde(rename = "message")]
    pub message: String,
}

impl BotMessage {
    pub fn new(message: String, index: usize) -> Self {
        Self {
            sender: "bot".to_owned(),
            index,
            message,
        }
    }
}
