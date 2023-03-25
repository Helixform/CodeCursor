use std::cell::Cell;

use futures::{stream, Stream, StreamExt};
use node_bridge::{
    http_client::{HttpMethod, HttpRequest, HttpResponse},
    prelude::console,
};
use wasm_bindgen::JsValue;

use crate::models::RequestBody;

pub struct ResponseState {
    response: HttpResponse,
    started: Cell<bool>,
    ended: Cell<bool>,
    first_newline_dropped: Cell<bool>,
    expect_begin_message: bool,
}

impl ResponseState {
    fn new(response: HttpResponse, expect_begin_message: bool) -> Self {
        Self {
            response,
            started: Cell::new(false),
            ended: Cell::new(false),
            first_newline_dropped: Cell::new(false),
            expect_begin_message,
        }
    }

    pub fn data_stream(&mut self) -> impl Stream<Item = String> + '_ {
        #[cfg(debug_assertions)]
        if !self.expect_begin_message {
            console::log_str(&format!("ignore begin message"));
        }
        self.started.set(!self.expect_begin_message);

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
                .filter(|s| {
                    if self.ended.get() {
                        return false;
                    }
                    if s == "<|BEGIN_message|>" {
                        self.started.set(true);
                        return false;
                    }
                    if s == "<|END_message|>" {
                        self.ended.set(true);
                        return false;
                    }
                    if !self.started.get() {
                        return false;
                    }
                    // Server may produce newlines at the head of response, we need
                    // to do this trick to ignore them in the final edit.
                    if !self.first_newline_dropped.get()
                        && s.trim().is_empty()
                        && self.expect_begin_message
                    {
                        self.first_newline_dropped.set(true);
                        return false;
                    }
                    true
                })
                .collect();
            stream::iter(lines)
        })
    }

    pub async fn complete(self) -> Result<(), JsValue> {
        self.response.await
    }
}

pub async fn make_request(
    path: &str,
    body: &RequestBody,
    expect_begin_message: bool,
) -> Result<ResponseState, JsValue> {
    let request = HttpRequest::new(&format!(
        "https://aicursor.com{}",
        path
    ))
    .set_method(HttpMethod::Post)
    .set_body(serde_json::to_string(&body).unwrap())
    .add_header("authority", "aicursor.com")
    .add_header("accept", "*/*")
    .add_header("content-type", "application/json")
    .add_header("user-agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Cursor/0.1.6 Chrome/108.0.5359.62 Electron/22.0.0 Safari/537.36");

    Ok(ResponseState::new(
        request.send().await?,
        expect_begin_message,
    ))
}
