pub mod stream;

use node_bridge::http_client::{HttpMethod, HttpRequest};
use serde::Serialize;

pub fn make_request(path: &str, method: HttpMethod) -> HttpRequest {
    HttpRequest::new(&format!("https://internal.cursor.sh{path}"))
        .set_method(method)
        .add_header("accept", "*/*")
        .add_header("content-type", "application/json")
        .add_header("user-agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Cursor/0.2.7 Chrome/102.0.5005.167 Electron/19.1.9 Safari/537.36")
}

pub trait JsonSendable {
    fn set_json_body<T>(self, body: &T) -> Self
    where
        T: Serialize;
}

impl JsonSendable for HttpRequest {
    fn set_json_body<T>(self, body: &T) -> Self
    where
        T: Serialize,
    {
        self.set_body(serde_json::to_string(body).unwrap())
    }
}
