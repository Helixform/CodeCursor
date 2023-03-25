use futures::StreamExt;
use node_bridge::{bindings::uuid, prelude::*};
use wasm_bindgen::JsValue;

use crate::{
    models::{user_message::UserMessage, BotMessage, MessageType, RequestBody},
    request::make_request,
    GenerateInput,
};

#[derive(Debug)]
pub struct Session {
    request_body: Option<RequestBody>,
    conversation_id: String,
}

impl Session {
    fn body_with_input(&mut self, input: &GenerateInput) -> &RequestBody {
        if self.request_body.is_none() {
            self.request_body = Some(RequestBody::new_with_input(input, MessageType::Freeform));
        }
        self.request_body.as_ref().unwrap()
    }

    fn push_bot_message(&mut self, message: String) {
        let bot_message = BotMessage::new(
            self.conversation_id.clone(),
            MessageType::Markdown,
            message,
            "<|END_message|>".to_owned(),
            "".to_owned(),
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
            conversation_id: uuid::uuid_v4(),
        }
    }

    pub async fn send_message(&mut self, input: &GenerateInput) -> Result<(), JsValue> {
        let request_body = self.body_with_input(input);
        let mut state = make_request("/conversation", request_body, true).await?;

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

        self.push_bot_message(message);

        Ok(())
    }
}
