// pub mod session;
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
use sha2::Digest;
use uuid::Uuid;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::future_to_promise;

use crate::{
    bindings::{progress_location::ProgressLocation, progress_options::ProgressOptions},
    context::get_extension_context,
    request::make_request,
};

use self::token::Token;

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
        .replace("+", "-")
        .replace("/", "_")
        .replace("=", "")
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
    let uuid = Uuid::new_v4().to_string();
    let verifier = base64_encode(random_bytes());
    let challenge = base64_encode(sha256(verifier.clone()));

    let login_url = format!(
        "https://cursor.so/loginDeepControl?challenge={challenge}&uuid={}",
        uuid.clone()
    );

    let context = get_extension_context();
    // The API of VSCode does not allow us to obtain the execution result of the 'vscode.open' command,
    // so we cannot determine whether the user has confirmed to open url.
    context
        .execute_command1("vscode.open", JsValue::from_str(&login_url))
        .await;

    context
        .with_progress(
            ProgressOptions {
                location: ProgressLocation::Notification,
                title: Some("Signing in...".to_owned()),
                cancellable: true,
            },
            closure!(|abort_signal: AbortSignal| {
                let uuid = uuid.clone();
                let verifier = verifier.clone();
                future_to_promise(async move {
                    polling(&uuid, &verifier, abort_signal)
                        .await
                        .map(Into::into)
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

    let mut interval = IntervalStream::new(2000);
    loop {
        let defer_abort_future = defer_abort.clone().into_future();
        match select(defer_abort_future, interval.next()).await {
            Either::Left(_) => {
                return Ok(None);
            }
            _ => {}
        }
        let mut response = make_request(
            &format!("/auth/poll?uuid={}&verifier={}", uuid, verifier),
            HttpMethod::Get,
        )
        .send()
        .await?;

        if let Some(chunk) = response.body().next().await {
            let data = chunk.to_string("utf-8");
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
}

pub fn sign_out() {}

#[cfg(test)]
mod test {
    #[test]
    fn test_random_bytes() {
        let bytes = super::random_bytes();
        assert_eq!(bytes.len(), 32);
    }
}
