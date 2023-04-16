use futures::StreamExt;
use node_bridge::prelude::*;
use uuid::Uuid;
use wasm_bindgen::JsValue;

use crate::{
    conversation::{
        make_conversation_request,
        models::{user_message::UserMessage, BotMessage, MessageType, RequestBody, UserRequest},
    },
    GenerateInput,
};

#[derive(Debug)]
pub struct Session {
    request_body: Option<RequestBody>,
    conversation_id: String,
}

impl Session {
    fn body_with_input(&mut self, input: &GenerateInput) -> &RequestBody {
        let message_type = MessageType::Freeform;

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
        let bot_message = BotMessage::new(
            self.conversation_id.clone(),
            MessageType::Markdown,
            message,
            "<|END_message|>".to_owned(),
            self.request_body
                .as_ref()
                .map(|r| r.user_request.current_root_path.clone())
                .unwrap_or_default(),
            true,
        );
        self.request_body
            .as_mut()
            .map(|r| r.bot_messages.push(bot_message));
    }

    fn push_user_message(&mut self, input: &GenerateInput) {
        let user_message =
            UserMessage::new_with_input(input, &self.conversation_id, MessageType::Freeform);
        self.request_body
            .as_mut()
            .map(|r| r.user_messages.push(user_message));
    }
}

impl Session {
    pub fn new() -> Self {
        Self {
            request_body: None,
            conversation_id: Uuid::new_v4().to_string(),
        }
    }

    pub async fn send_message(&mut self, input: &GenerateInput) -> Result<(), JsValue> {
        let request_body = self.body_with_input(input);
        let mut state = make_conversation_request("/conversation", request_body).await?;

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

        self.push_user_message(input);
        self.push_bot_message(message);

        Ok(())
    }
}

impl Default for Session {
    fn default() -> Self {
        Self::new()
    }
}
