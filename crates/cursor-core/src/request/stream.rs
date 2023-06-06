use std::vec;

use futures::{future, stream, Stream, StreamExt};
use node_bridge::{
    http_client::{Http2Response, HttpMethod, HttpRequest, HttpResponse},
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

pub struct Http2StreamResponseState {
    response: Http2Response,
}

impl Http2StreamResponseState {
    fn new(response: Http2Response) -> Self {
        Self { response }
    }

    pub fn data_stream(&mut self) -> impl Stream<Item = String> + '_ {
        let mut last_message_leftover_length = 0u32;
        let mut all_message_drained = false;
        let mut cache_chunk = Vec::<u8>::new();

        self.response
            .body()
            .map(move |chunk| {
                if all_message_drained {
                    return vec![];
                }

                let mut resolved_chunk = Vec::<u8>::new();
                cache_chunk.extend_from_slice(chunk.as_slice());

                loop {
                    
                    if last_message_leftover_length > 0 {
                        let usize_last_message_leftover_length: usize =
                            last_message_leftover_length.try_into().unwrap();
                        if cache_chunk.len() >= usize_last_message_leftover_length {
                            resolved_chunk
                                .extend_from_slice(&cache_chunk[0..usize_last_message_leftover_length]);
                            last_message_leftover_length = 0;
                            cache_chunk.splice(0..usize_last_message_leftover_length, vec![]);
                        } else {
                            let len: u32 = cache_chunk.len().try_into().unwrap();
                            resolved_chunk.append(&mut cache_chunk);
                            last_message_leftover_length -= len;
                        }
                    }

                    if cache_chunk.len() == 0 {
                        break;
                    }

                    if last_message_leftover_length == 0 {
                        if cache_chunk.len() >= 5 {
                            if cache_chunk[0] == 2 {
                                all_message_drained = true;
                                break;
                            }

                            let parts = std::array::from_fn::<u8, 4, _>(|i| cache_chunk[i + 1]);
                            last_message_leftover_length = u32::from_be_bytes(parts);
                            cache_chunk.splice(0..5, vec![]);
                        } else {
                            break;
                        }
                    }
                }
                resolved_chunk
            })
            .filter(|chunk| future::ready(chunk.len() > 0))
            .map(|chunk| String::from_utf8(chunk).unwrap_or_default())
    }

    pub async fn complete(self) -> Result<(), JsValue> {
        self.response.await
    }
}

impl From<Http2Response> for Http2StreamResponseState {
    fn from(response: Http2Response) -> Self {
        Self::new(response)
    }
}
