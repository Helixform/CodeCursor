use futures::{stream, Stream, StreamExt};
use node_bridge::{
    http_client::{HttpMethod, HttpRequest, HttpResponse},
    prelude::*,
};
use serde::Serialize;
use serde_json::{Result as JsonResult, Value};
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
                    if l.len() > 0 && l.starts_with("data: ") {
                        // 1. 用slice方法获取"data: "之后的部分
                        let suffix = &l["data: ".len()..];
                        let parse_ret: JsonResult<Value> = serde_json::from_str(suffix);

                        // 2. 判断是否解析成功
                        if parse_ret.is_err() {
                            // 2-1. 如果失败，则直接返回 如 [DONE]
                            console::log_str("-------2-1------");
                            console::log_str(&suffix);
                            None
                            // if suffix.starts_with("[DONE]") {
                            //     console::log_str("-----我已经 starts_with了------");
                            //     Some(String::from(suffix))
                            // } else {
                            //     None
                            // }
                        } else {
                            // 2-2. 如果成功,则取一下值，取的到则返回值，否则也是返回None
                            let v_data: Value = parse_ret.unwrap();
                            let str_content = v_data["choices"][0]["delta"]["content"].to_string();
                            let ret_content = str_content.trim_matches('\"');
                            console::log_str("-------2-2------");
                            // let ret = input.trim_matches('\"');
                            console::log_str(str_content.as_str());
                            console::log_str(ret_content);
                            if str_content == "null" {
                                None
                            } else {
                                Some(ret_content.to_string())
                            }
                        }
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
