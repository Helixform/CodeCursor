use std::result;

use futures::StreamExt;
use node_bridge::prelude::*;
use wasm_bindgen::JsValue;
use regex::Regex;

use crate::{
    conversation::{
        models::{user_message::UserMessage, BotMessage, MessageType, RequestBody, ConversationRequestBody, UserRequest},
        send_http2_conversation_request,
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

    fn conversation_body_with_input(&mut self, input: &GenerateInput) -> ConversationRequestBody {
        input.into()
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
        let request_body = self.conversation_body_with_input(input);
        #[cfg(debug_assertions)]
        console::log_str(&serde_json::to_string(&request_body).unwrap());

        let mut state = send_http2_conversation_request(&request_body).await?;

        let mut message: String = "".to_owned();

        let mut data_stream = state.data_stream();
        let result_stream = input.result_stream();
        while let Some(data) = data_stream.next().await {
            #[cfg(debug_assertions)]
            console::log_str(&data);
            message.push_str(&data);
        }
        drop(data_stream);

        state.complete().await?;

        self.push_user_message(input.prompt());

        #[cfg(debug_assertions)]
        console::log_str(&format!("raw message: {}", message));

        let re = Regex::new(r#"\{"text":"((?s).*?)"\}"#).unwrap();
        
        let mut resolved_messages = Vec::<&str>::new(); 
        for value in re.captures_iter(&message) {
            let v = value.get(1).unwrap().as_str();

            #[cfg(debug_assertions)]
            console::log_str(&format!("matched message: {}", v));

            result_stream.write( &v.replace("\\n", "\n").replace(r#"\""#, "\""));
           
            //result_stream.write(v);
            resolved_messages.push(v);
        }

        result_stream.end();

        let resolved_message = resolved_messages.join("").to_string();
        self.push_bot_message(resolved_message);

        Ok(())
    }
}

impl Default for Session {
    fn default() -> Self {
        Self::new()
    }
}
