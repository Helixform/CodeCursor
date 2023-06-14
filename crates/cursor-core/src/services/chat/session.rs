use std::pin::pin;

use futures::StreamExt;
use node_bridge::prelude::*;
use wasm_bindgen::{JsError, JsValue};

use crate::{
    services::{
        enveloped_message::{FilledPrompt, MessageContent},
        stream::make_stream,
    },
    GenerateInput,
};

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
        self.request_body = Some(
            self.request_body
                .take()
                .map(|mut r| {
                    r.conversation
                        .push(ConversationMessage::new(MessageType::User, input.prompt()));
                    r
                })
                .unwrap_or_else(|| RequestBody::new_with_input(input)),
        )
        .map(|mut r| {
            // Add an empty bot message to the conversation.
            r.conversation
                .push(ConversationMessage::empty_message(MessageType::Bot));
            r
        });
        self.request_body.as_ref().unwrap()
    }

    fn push_bot_message(&mut self, message: String) {
        self.request_body.as_mut().map(|r| {
            r.conversation.last_mut().map(|m| {
                if let MessageType::Bot = m.message_type {
                    m.text = message
                }
            })
        });
    }
}

impl Session {
    pub fn new() -> Self {
        Self { request_body: None }
    }

    pub async fn send_message(&mut self, input: &GenerateInput) -> Result<(), JsValue> {
        let request_body = self.body_with_input(input);

        #[cfg(debug_assertions)]
        console::log_str(&serde_json::to_string_pretty(request_body).unwrap());

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
                if let Ok(prompt) = serde_json::from_str::<FilledPrompt>(&data) {
                    #[cfg(debug_assertions)]
                    console::log_str(&format!("prompt: \n{}", prompt.text));
                    continue;
                } else if let Ok(MessageContent { text, .. }) =
                    serde_json::from_str::<MessageContent>(&data)
                {
                    #[cfg(debug_assertions)]
                    console::log_str(&format!("wrote: {}", text));
                    result_stream.write(&text);
                    message.push_str(&data);
                }
            }
        }

        state.complete().await?;
        result_stream.end();

        self.push_bot_message(message);

        Ok(())
    }
}

impl Default for Session {
    fn default() -> Self {
        Self::new()
    }
}
