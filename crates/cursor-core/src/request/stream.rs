use futures::{Stream, StreamExt};
use js_sys::Uint8Array;
use node_bridge::http_client::HttpResponse;

use wasm_bindgen::prelude::*;

pub struct StreamResponseState {
    response: HttpResponse,
}

impl StreamResponseState {
    fn new(response: HttpResponse) -> Self {
        Self { response }
    }

    pub fn data_stream(&mut self) -> impl Stream<Item = Vec<u8>> + '_ {
        self.response
            .body()
            .map(|chunk| Uint8Array::new(&chunk).to_vec())
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
