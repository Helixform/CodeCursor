pub mod models;

use futures::{Stream, StreamExt};
use js_sys::Uint8Array;
use node_bridge::http_client::{HttpMethod, HttpResponse};
use serde::Serialize;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;

use crate::{
    auth::account_token,
    context::get_extension_context,
    request::{make_request, API2_HOST},
};

use super::enveloped_message::EnvelopedMessage;

const SIGN_IN_ITEM: &str = "Sign In / Sign Up";
const CONFIGURE_API_KEY_ITEM: &str = "Configure API Key";

pub struct StreamResponseState {
    response: HttpResponse,
}

impl StreamResponseState {
    fn new(response: HttpResponse) -> Self {
        Self { response }
    }

    pub fn data_stream(&mut self) -> impl Stream<Item = EnvelopedMessage> + '_ {
        self.response.body().filter_map(|chunk| async move {
            let bytes = Uint8Array::new(&chunk).to_vec();
            EnvelopedMessage::decode(bytes).ok()
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

pub async fn make_stream<T>(path: &str, body: &T) -> Result<StreamResponseState, JsValue>
where
    T: Serialize,
{
    let mut request = make_request(API2_HOST, path, HttpMethod::Post)
        .add_header("content-type", "application/connect+json");

    let context = get_extension_context();
    if let Some(token) = account_token() {
        request = request.add_header("Authorization", &format!("Bearer {}", token.access_token));
    } else if context.model_configuration().api_key().is_none() {
        spawn_local(async move {
            if let Some(pick) = context
                .show_information_message(
                    "You have to sign in / sign up or configure API key to use Cursor AI features",
                    [SIGN_IN_ITEM, CONFIGURE_API_KEY_ITEM]
                        .into_iter()
                        .map(JsValue::from)
                        .collect(),
                )
                .await
                .as_string()
            {
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
        });
        return Err(JsError::new("No API key or account token").into());
    }

    let chunk = EnvelopedMessage::new_with_serializable(&body, 0)
        .map_err(|e| JsError::new(&e.to_string()))?;
    // The data will always end with an empty data block flagged as 2.
    let body = [chunk, EnvelopedMessage::end()]
        .into_iter()
        .map(|d| d.encode())
        .reduce(|a, b| {
            let mut result = a;
            result.extend_from_slice(&b);
            result
        });
    let response = request.set_body(body).send().await?;
    if response.status_code() != 200 {
        return Err(js_sys::Error::new(&format!(
            "Server returned status code {}",
            response.status_code()
        ))
        .into());
    }
    Ok(response.into())
}
