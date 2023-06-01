use std::pin::pin;

use futures::StreamExt;
use node_bridge::prelude::*;
use wasm_bindgen::{JsError, JsValue};

use crate::{services::stream::make_stream, GenerateInput};

use super::models::{
    conversation::{ConversationMessage, MessageType},
    request_body::RequestBody,
};

#[derive(Debug)]
pub struct Session {
    request_body: Option<RequestBody>,
}

impl Session {
    fn body_with_input(&mut self, input: &GenerateInput) -> &RequestBody {
        self.request_body = Some(RequestBody::new_with_input(input));
        self.request_body.as_ref().unwrap()
    }

    fn push_message(&mut self, msg_type: MessageType, message: String) {
        let message = ConversationMessage::new(msg_type, message);
        self.request_body
            .as_mut()
            .map(|r| r.conversation.push(message));
    }

    fn push_bot_message(&mut self, message: String) {
        self.push_message(MessageType::Bot, message);
    }

    fn push_user_message(&mut self, message: String) {
        self.push_message(MessageType::Human, message);
    }
}

impl Session {
    pub fn new() -> Self {
        Self { request_body: None }
    }

    pub async fn send_message(&mut self, input: &GenerateInput) -> Result<(), JsValue> {
        let request_body = self.body_with_input(input);
        #[cfg(debug_assertions)]
        console::log_str(&serde_json::to_string(&request_body).unwrap());
        let mut state = make_stream("/aiserver.v1.AiService/StreamChat", request_body).await?;
        let result_stream = input.result_stream();

        let mut message = String::new();
        {
            let mut data_stream = pin!(state.data_stream());
            while let Some(chunk) = data_stream.next().await {
                if chunk.is_end() {
                    break;
                }
                let data = chunk
                    .utf8_string()
                    .map_err(|e| JsError::new(&e.to_string()))?;
                if data.is_empty() {
                    continue;
                }
                #[cfg(debug_assertions)]
                console::log_str(&data);
                // result_stream.write(&data);
                // message.push_str(&data);
            }
        }

        state.complete().await?;
        result_stream.end();

        self.push_user_message(input.prompt());
        self.push_bot_message(message);

        Ok(())
    }
}

impl Default for Session {
    fn default() -> Self {
        Self::new()
    }
}
