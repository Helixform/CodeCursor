pub mod chat;
pub mod generate;
pub mod models;

use node_bridge::{http_client::HttpMethod, prelude::console};
use wasm_bindgen::{JsError, JsValue};
use wasm_bindgen_futures::spawn_local;

use crate::{
    auth::account_token,
    context::get_extension_context,
    request::{
        make_request_more_freedom, make_request_with_legacy,
        stream::{make_stream_request, StreamResponseState, Http2StreamResponseState},
        JsonSendable,
    },
};

use self::models::{ConversationRequestBody, RequestBody};

const SIGN_IN_ITEM: &str = "Sign In / Sign Up";
const CONFIGURE_API_KEY_ITEM: &str = "Configure API Key";

async fn send_conversation_request(
    path: &str,
    body: &RequestBody,
) -> Result<StreamResponseState, JsValue> {
    // for /conversation api call, we must use "aicursor.com" host, so 
    // we should call make_request_with_legacy whose legacy_host arg is false.
    let mut request =  make_request_with_legacy(path, HttpMethod::Post, false).set_json_body(body);
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


async fn send_http2_conversation_request(
    body: &ConversationRequestBody,
) -> Result<Http2StreamResponseState, JsValue> {
    // encode the body as envelop request message following 
    // [connect protocol](https://connect.build/docs/protocol)

    let enveloped_request_message = body.to_enveloped_message();

    // in past, for /conversation api call, we must use "aicursor.com" host, so
    // we should call make_request_with_legacy whose legacy_host arg is false.
    // and this way is deprecated!
    //
    // instead, following new edition of cursor, use "https://api2.cursor.sh/aiserver.v1.AiService/StreamChat"

    // this request is http2 protocol + https

    let mut request = make_request_more_freedom(
        "api2.cursor.sh",
        "/aiserver.v1.AiService/StreamChat",
        HttpMethod::Post,
    )
    .add_header(":path", "/aiserver.v1.AiService/StreamChat")
    .add_header(":method", "POST")
    .add_header(":scheme", "https")
    .add_header(":authority", "api2.cursor.sh")
    .add_header("connect-protocol-version", "1")
    .add_header("content-type", "application/connect+json")
    .add_header(
        "content-length",
        enveloped_request_message.len().to_string().as_str(),
    )
    .add_header("accept-encoding", "gzip, deflate, br")
    .set_body_bytes(enveloped_request_message);

    // let mut request =  make_request_with_legacy(path, HttpMethod::Post, false).set_json_body(body);

    if body.model_details.api_key.is_none() {
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
    
    #[cfg(debug_assertions)]
    console::log_str("before request http2");

    let response = request.send_as_http2().await?;
    Ok(response.into())

}