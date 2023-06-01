use futures::StreamExt;
use node_bridge::prelude::*;
use wasm_bindgen::JsValue;

use crate::{
    conversation::{
        models::{user_message::UserMessage, BotMessage, MessageType, RequestBody, UserRequest},
        send_conversation_request,
    },
    GenerateInput,
};

#[derive(Debug)]
pub struct Session {
    request_body: Option<RequestBody>,
    message_counter: usize,
}

impl Session {
    fn body_with_input(&mut self, input: &GenerateInput) -> &RequestBody {
        let str_msg_type = input.msg_type();
        let message_type = match str_msg_type.as_str() {
            "Freeform" => MessageType::Freeform,
            "Edit" => MessageType::Edit,
            "Custom" => MessageType::Custom,
            "Generate" => MessageType::Generate,
            "GenVar" => MessageType::GenVar,
            _ => MessageType::Custom,
        };

        self.request_body = Some(
            self.request_body
                .take()
                .map(|mut r| {
                    r.user_request = UserRequest::new_with_input(input, message_type);
                    r
                })
                .unwrap_or_else(|| RequestBody::new_with_input(input, message_type)),
        );
        self.request_body.as_ref().unwrap()
    }

    fn push_bot_message(&mut self, message: String) {
        let bot_message = BotMessage::new(message, self.message_counter);
        self.request_body
            .as_mut()
            .map(|r| r.bot_messages.push(bot_message));
        self.message_counter += 1;
    }

    fn push_user_message(&mut self, message: String) {
        let user_message = UserMessage::new(message, self.message_counter);
        self.request_body
            .as_mut()
            .map(|r| r.user_messages.push(user_message));
        self.message_counter += 1;
    }
}

impl Session {
    pub fn new() -> Self {
        Self {
            request_body: None,
            message_counter: 0,
        }
    }

    pub async fn send_message(&mut self, input: &GenerateInput) -> Result<(), JsValue> {
        let request_body = self.body_with_input(input);
        #[cfg(debug_assertions)]
        console::log_str(&serde_json::to_string(&request_body).unwrap());
        let mut state = send_conversation_request("/conversation", request_body).await?;

        let mut message: String = "".to_owned();

        let mut data_stream = state.data_stream();
        let result_stream = input.result_stream();
        while let Some(data) = data_stream.next().await {
            #[cfg(debug_assertions)]
            console::log_str(&data);
            result_stream.write(&data);
            message.push_str(&data);
        }
        drop(data_stream);

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
