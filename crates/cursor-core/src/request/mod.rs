pub mod stream;

use node_bridge::http_client::{HttpMethod, HttpRequest};
use serde::Serialize;

pub const API2_HOST: &str = "api2.cursor.sh";
pub const INTERNAL_HOST: &str = "internal.cursor.sh";

pub fn make_request(host: &str, path: &str, method: HttpMethod) -> HttpRequest {
    HttpRequest::new(&format!("https://{host}{path}"))
        .set_method(method)
        .add_header("accept", "*/*")
        .add_header("user-agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Cursor/0.2.32 Chrome/102.0.5005.167 Electron/19.1.9 Safari/537.36")
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
        self.add_header("content-type", "application/json")
            .set_body(serde_json::to_string(body).ok())
    }
}
