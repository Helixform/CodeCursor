pub mod chat;
pub mod generate;
pub mod models;

use futures::{stream, Stream, StreamExt};
use node_bridge::{
    http_client::{HttpMethod, HttpResponse},
    prelude::console,
};
use wasm_bindgen::{JsError, JsValue};
use wasm_bindgen_futures::spawn_local;

use crate::{
    auth::account_token,
    context::get_extension_context,
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

const SIGN_IN_ITEM: &str = "Sign In / Sign Up";
const CONFIGURE_API_KEY_ITEM: &str = "Configure API Key";

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
            spawn_local(async move {
                let context = get_extension_context();
                match context
                    .show_information_message(
                        "You have to sign in / sign up or configure API key to use Cursor AI feature",
                        vec![SIGN_IN_ITEM, CONFIGURE_API_KEY_ITEM]
                            .into_iter()
                            .map(JsValue::from)
                            .collect(),
                    )
                    .await
                    .as_string()
                {
                    Some(pick) => {
                        context
                            .execute_command0(&format!(
                                "aicursor.{}",
                                if pick == SIGN_IN_ITEM {
                                    "signInUp"
                                } else {
                                    "configureApiKey"
                                }
                            ))
                            .await;
                    }
                    None => {}
                }
            });
            return Err(JsError::new("No API key or account token").into());
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
