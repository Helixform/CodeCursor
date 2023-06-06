pub mod stream;

use node_bridge::http_client::{HttpMethod, HttpRequest};
use serde::Serialize;

/// Make a request to the legacy host.
///
/// Due to the higher version of Cursor modifying the API's host,
/// but we found that the authentication interface has not been modified.
///
/// We believe that Cursor will probably unify the host in the future, so this function is used for compatibility.
pub fn make_request_with_legacy(path: &str, method: HttpMethod, legacy_host: bool) -> HttpRequest {
    // I open the Cursor App and observe its http request, 
    // and make some experiments, find these rules:
    // - for /auth/poll, /gen_project, use "internal.cursor.sh"
    // - for /conversation, use "aicursor.com"
    let host = if legacy_host {
        "aicursor.com"
    } else {
        "internal.cursor.sh"
    };

    HttpRequest::new(&format!("https://{host}{path}"))
        .set_method(method)
        .add_header("accept", "*/*")
        .add_header("content-type", "application/json")
        .add_header("user-agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Cursor/0.2.7 Chrome/102.0.5005.167 Electron/19.1.9 Safari/537.36")
}

pub fn make_request(path: &str, method: HttpMethod) -> HttpRequest {
    make_request_with_legacy(path, method, false)
}

pub fn make_request_more_freedom(host: &str, path: &str, method: HttpMethod) -> HttpRequest {
    HttpRequest::new(&format!("https://{host}{path}"))
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
