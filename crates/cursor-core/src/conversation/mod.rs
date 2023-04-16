pub mod chat;
pub mod generate;
pub mod models;

use std::cell::Cell;

use futures::{stream, Stream, StreamExt};
use node_bridge::{
    http_client::{HttpMethod, HttpResponse},
    prelude::console,
};
use wasm_bindgen::JsValue;

use crate::{
    auth::account_token,
    request::{make_request, JsonSendable},
};

use self::models::RequestBody;

struct ResponseState {
    response: HttpResponse,
}

impl ResponseState {
    fn new(response: HttpResponse) -> Self {
        Self { response }
    }

    pub fn data_stream(&mut self) -> impl Stream<Item = String> + '_ {
        self.response.body().flat_map(|chunk| {
            let chunk = chunk.to_string("utf-8");
            #[cfg(debug_assertions)]
            console::log_str(&chunk);

            let lines: Vec<_> = chunk
                .split("\n")
                .filter_map(|l| {
                    if l.len() > 0 && l.starts_with("data: \"") {
                        serde_json::from_str::<String>(&l["data: ".len()..]).ok()
                    } else {
                        None
                    }
                })
                .filter(|s| s != "[DONE]")
                .collect();
            stream::iter(lines)
        })
    }

    pub async fn complete(self) -> Result<(), JsValue> {
        self.response.await
    }
}

async fn make_conversation_request(
    path: &str,
    body: &RequestBody,
) -> Result<ResponseState, JsValue> {
    let mut request = make_request(path, HttpMethod::Post);
    if body.api_key.is_none() {
        if let Some(token) = account_token() {
            request =
                request.add_header("Authorization", &format!("Bearer {}", token.access_token));
        } else {
            todo!()
        }
    }
    let response = request.set_json_body(&body).send().await?;
    if response.status_code() != 200 {
        return Err(js_sys::Error::new(&format!(
            "Server returned status code {}",
            response.status_code()
        ))
        .into());
    }
    Ok(ResponseState::new(response))
}
