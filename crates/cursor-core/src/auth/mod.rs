pub mod token;

use std::future::IntoFuture;

use base64::Engine;
use futures::{
    future::{select, Either},
    StreamExt,
};
use gloo::timers::future::IntervalStream;
use node_bridge::{bindings::AbortSignal, futures::Defer, http_client::HttpMethod, prelude::*};
use rand::RngCore;
use serde::Deserialize;
use serde_json::json;
use sha2::Digest;
use uuid::Uuid;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::{future_to_promise, spawn_local};

use crate::{
    bindings::{
        progress::Progress, progress_location::ProgressLocation, progress_options::ProgressOptions,
    },
    context::get_extension_context,
    request::{make_request, JsonSendable, API2_HOST},
};

use self::token::Token;

const AUTH_TOKEN_KEY: &str = "auth_token";
const CLIENT_ID: &str = "KbZUR41cY7W6zRSdpSUJ7I7mLYBKOCmB";

fn random_bytes() -> Vec<u8> {
    let mut rng = rand::thread_rng();
    let mut bytes = vec![0u8; 32];
    let _ = rng.try_fill_bytes(&mut bytes);
    bytes
}

fn base64_encode<T>(bytes: T) -> String
where
    T: AsRef<[u8]>,
{
    base64::engine::general_purpose::STANDARD
        .encode(bytes)
        .replace('+', "-")
        .replace('/', "_")
        .replace('=', "")
}

fn sha256<T>(data: T) -> Vec<u8>
where
    T: AsRef<[u8]>,
{
    let mut hasher = sha2::Sha256::new();
    hasher.update(data);
    hasher.finalize().to_vec()
}

#[wasm_bindgen(js_name = signIn)]
pub async fn sign_in() {
    let context = get_extension_context();
    let storage = context.storage();
    if storage.get(AUTH_TOKEN_KEY).is_some() {
        // If there is already an authentication token, it means that the user has logged in
        // and does not need to log in again.
        context
            .show_information_message("You have already logged in.", js_sys::Array::new())
            .await;
        return;
    }

    let uuid = Uuid::new_v4().to_string();
    let verifier = base64_encode(random_bytes());
    let challenge = base64_encode(sha256(verifier.clone()));

    let login_url = format!(
        "https://cursor.sh/loginDeepControl?challenge={challenge}&uuid={}",
        uuid.clone()
    );

    // The API of VSCode does not allow us to obtain the execution result of the 'vscode.open' command,
    // so we cannot determine whether the user has confirmed to open url.
    context
        .execute_command1("vscode.open", JsValue::from_str(&login_url))
        .await;

    context
        .with_progress(
            ProgressOptions {
                location: ProgressLocation::Notification,
                title: Some("Waiting for sign in / sign up...".to_owned()),
                cancellable: true,
            },
            closure_once!(|_progress: Progress, abort_signal: AbortSignal| {
                future_to_promise(async move {
                    Ok(
                        if let Some(token) = polling(&uuid, &verifier, abort_signal).await? {
                            storage.update(AUTH_TOKEN_KEY, Some(&token));
                            spawn_local(async move {
                                get_extension_context()
                                    .show_information_message(
                                        "You have successfully logged in.",
                                        js_sys::Array::new(),
                                    )
                                    .await;
                            });
                            JsValue::from_str(&token)
                        } else {
                            JsValue::null()
                        },
                    )
                })
            })
            .into_js_value()
            .into(),
        )
        .await;
}

async fn polling(
    uuid: &str,
    verifier: &str,
    abort_signal: AbortSignal,
) -> Result<Option<String>, JsValue> {
    let defer_abort = Defer::new();
    let defer_abort_clone = defer_abort.clone();
    abort_signal.add_event_listener(
        "abort",
        closure_once!(|| {
            defer_abort_clone.resolve(JsValue::null());
        })
        .into_js_value(),
    );

    let mut interval = IntervalStream::new(1000);
    loop {
        let defer_abort_future = defer_abort.clone().into_future();
        if let Either::Left(_) = select(defer_abort_future, interval.next()).await {
            return Ok(None);
        }
        let Ok(mut response) = make_request(
            API2_HOST,
            &format!("/auth/poll?uuid={uuid}&verifier={verifier}"),
            HttpMethod::Get,
        )
        .send()
        .await
        else {
            continue;
        };
        if response.status_code() != 200 {
            // If the request fails, it means that the server is not ready yet,
            // so we need to continue polling.
            continue;
        }
        let data = response.text().await;

        #[cfg(debug_assertions)]
        console::log_str(&data);
        match serde_json::from_str::<serde_json::Value>(&data).and_then(|value| {
            if value.is_null() {
                Ok(false)
            } else {
                serde_json::from_str::<Token>(&data).map(|_| true)
            }
        }) {
            Ok(flag) => {
                if !flag {
                    continue;
                }
                return Ok(Some(data));
            }
            Err(err) => {
                let js_error = JsError::new(&err.to_string());
                let error = js_error.into();
                #[cfg(debug_assertions)]
                console::error1(&error);
                return Err(error);
            }
        }
    }
}

#[wasm_bindgen(js_name = signOut)]
pub fn sign_out() {
    let context = get_extension_context();
    context.storage().update(AUTH_TOKEN_KEY, None);
    spawn_local(async move {
        context
            .show_information_message("You have successfully logged out.", js_sys::Array::new())
            .await;
    });
}

#[derive(Debug, Clone, Deserialize)]
struct RefreshResponse {
    pub access_token: String,
    #[serde(rename = "expires_in")]
    pub _expires_in: u64,
    #[serde(rename = "scope")]
    pub _scope: String,
    #[serde(rename = "token_type")]
    pub _token_type: String,
    #[serde(rename = "id_token")]
    pub _id_token: String,
}

#[wasm_bindgen(js_name = refreshToken)]
pub async fn refresh() -> Result<(), JsValue> {
    if let Some(mut token) = account_token() {
        let response = make_request("cursor.us.auth0.com", "/oauth/token", HttpMethod::Post)
            .set_json_body(&json!({
                "client_id": CLIENT_ID,
                "grant_type": "refresh_token",
                "refresh_token": token.refresh_token,
            }))
            .send()
            .await?
            .text()
            .await;
        #[cfg(debug_assertions)]
        console::log_str(&format!("refresh token response: {response}"));
        let access_token = serde_json::from_str::<RefreshResponse>(&response)
            .map_err(JsError::from)?
            .access_token;
        token.access_token = access_token;

        let context = get_extension_context();
        let storage = context.storage();
        storage.update(
            AUTH_TOKEN_KEY,
            Some(&serde_json::to_string(&token).map_err(JsError::from)?),
        );
    }
    Ok(())
}

pub fn account_token() -> Option<Token> {
    get_extension_context()
        .storage()
        .get(AUTH_TOKEN_KEY)
        .and_then(|token| serde_json::from_str(&token).ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_random_bytes() {
        let bytes = super::random_bytes();
        assert_eq!(bytes.len(), 32);
    }

    #[test]
    fn test_base64_encode() {
        let bytes = vec![
            0xa9, 0x1e, 0x74, 0x36, 0x4a, 0x57, 0xb6, 0x40, 0xcf, 0x25, 0x37, 0xf7, 0x20, 0x26,
            0x7a, 0x2e, 0x94, 0x90, 0x03, 0x85, 0x5b, 0xb8, 0xd0, 0x92, 0x37, 0xdc, 0xb3, 0xd9,
            0x0a, 0x4d, 0xd4, 0xc5,
        ];
        let encoded = base64_encode(bytes);
        assert_eq!(encoded, "qR50NkpXtkDPJTf3ICZ6LpSQA4VbuNCSN9yz2QpN1MU");
    }

    #[test]
    fn test_sha256() {
        let v = "qR50NkpXtkDPJTf3ICZ6LpSQA4VbuNCSN9yz2QpN1MU";
        assert_eq!(
            base64_encode(sha256(v)),
            "ddiNacYgAjUZTDf6Pza1wRlSjuWIQRz5Z1Jc2Bj4DII"
        );
    }
}
