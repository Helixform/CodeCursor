use futures::{stream, Stream, StreamExt};
use node_bridge::{
    http_client::{HttpMethod, HttpRequest, HttpResponse},
    prelude::*,
};
use serde::Serialize;
use wasm_bindgen::prelude::*;

use super::{make_request, JsonSendable};

pub struct StreamResponseState {
    response: HttpResponse,
}

impl StreamResponseState {
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

impl From<HttpResponse> for StreamResponseState {
    fn from(response: HttpResponse) -> Self {
        Self::new(response)
    }
}

pub fn make_stream_request<B>(path: &str, body: &B) -> HttpRequest
where
    B: Serialize,
{
    make_request(path, HttpMethod::Post).set_json_body(body)
}
