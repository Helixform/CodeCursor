use node_bridge::http_client::{HttpMethod, HttpRequest, HttpResponse};
use serde::Serialize;
use wasm_bindgen::JsValue;

pub async fn make_request<B>(
    path: &str,
    body: &B,
    method: HttpMethod,
) -> Result<HttpResponse, JsValue>
where
    B: Serialize,
{
    let user_agent = &format!("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Cursor/0.2.0 Chrome/102.0.5005.167 Electron/19.1.9 Safari/537.36");
    let request = HttpRequest::new(&format!("https://aicursor.com{path}"))
        .set_method(method)
        .set_body(serde_json::to_string(&body).unwrap())
        .add_header("authority", "aicursor.com")
        .add_header("accept", "*/*")
        .add_header("content-type", "application/json")
        .add_header("user-agent", user_agent);

    request.send().await
}
