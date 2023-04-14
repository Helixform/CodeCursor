use std::future::IntoFuture;

use base64::Engine;
use futures::{
    future::{select, Either},
    StreamExt,
};
use gloo::timers::future::IntervalStream;
use js_sys::Promise;
use node_bridge::{
    bindings::AbortSignal, closure_once, futures::Defer, http_client::HttpMethod, prelude::console,
};
use sha2::Digest;
use uuid::Uuid;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;

use crate::{auth::random_bytes, request::make_request};

use super::token::Token;

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

#[wasm_bindgen(getter_with_clone)]
pub struct Session {
    pub login_url: String,
    pub token: Promise,
}

impl Session {
    pub fn new(abort_signal: AbortSignal) -> Self {
        let uuid = Uuid::new_v4().to_string();
        let verifier = base64_encode(random_bytes());
        let challenge = base64_encode(sha256(verifier.clone()));
        let defer_abort = Defer::new();
        let defer_abort_clone = defer_abort.clone();
        abort_signal.add_event_listener(
            "abort",
            closure_once!(|| {
                defer_abort_clone.resolve(JsValue::null());
            })
            .into_js_value(),
        );
        Self {
            login_url: format!(
                "https://cursor.so/loginDeepControl?challenge={challenge}&uuid={}",
                uuid.clone()
            )
            .to_owned(),
            token: Promise::new(&mut |resolve, reject| {
                let uuid = uuid.clone();
                let verifier = verifier.clone();
                let defer_abort = defer_abort.clone();
                spawn_local(async move {
                    let mut interval = IntervalStream::new(2000);
                    loop {
                        let defer_abort_future = defer_abort.clone().into_future();
                        match select(defer_abort_future, interval.next()).await {
                            Either::Left(_) => {
                                let _ = resolve.call1(&JsValue::null(), &JsValue::null());
                                return;
                            }
                            _ => {}
                        }
                        if let Ok(mut response) = make_request(
                            &format!("/auth/poll?uuid={uuid}&verifier={verifier}"),
                            HttpMethod::Get,
                        )
                        .send()
                        .await
                        {
                            if let Some(chunk) = response.body().next().await {
                                let data = chunk.to_string("utf-8");
                                #[cfg(debug_assertions)]
                                console::log_str(&data);
                                match serde_json::from_str::<serde_json::Value>(&data).and_then(
                                    |value| {
                                        if value.is_null() {
                                            Ok(false)
                                        } else {
                                            serde_json::from_str::<Token>(&data).map(|_| true)
                                        }
                                    },
                                ) {
                                    Ok(flag) => {
                                        if !flag {
                                            continue;
                                        }
                                        let _ = resolve
                                            .call1(&JsValue::null(), &JsValue::from_str(&data));
                                    }
                                    Err(err) => {
                                        let js_error = JsError::new(&err.to_string());
                                        let error = js_error.into();
                                        #[cfg(debug_assertions)]
                                        console::error1(&error);
                                        let _ = reject.call1(&JsValue::null(), &error);
                                    }
                                }
                                return;
                            }
                            let _ = response.await;
                        }
                    }
                })
            }),
        }
    }

    pub fn cancel_polling(&self) {}
}

#[cfg(test)]
mod test {
    use super::*;

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
