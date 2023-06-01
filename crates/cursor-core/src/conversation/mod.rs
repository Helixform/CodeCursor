pub mod models;

use wasm_bindgen::{JsError, JsValue};
use wasm_bindgen_futures::spawn_local;

use crate::{
    auth::account_token,
    context::get_extension_context,
    request::{
        stream::{make_stream_request, StreamResponseState},
        JsonSendable,
    },
};

use self::models::RequestBody;

const SIGN_IN_ITEM: &str = "Sign In / Sign Up";
const CONFIGURE_API_KEY_ITEM: &str = "Configure API Key";

async fn send_conversation_request(
    path: &str,
    body: &RequestBody,
) -> Result<StreamResponseState, JsValue> {
    let mut request = make_stream_request(path, body);
    if body.api_key.is_none() {
        if let Some(token) = account_token() {
            request =
                request.add_header("Authorization", &format!("Bearer {}", token.access_token));
        } else {
            spawn_local(async move {
                let context = get_extension_context();
                match context
                    .show_information_message(
                        "You have to sign in / sign up or configure API key to use Cursor AI features",
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
    Ok(response.into())
}
