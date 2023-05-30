use node_bridge::bindings::TextEncoder;
use serde::Serialize;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;

use crate::{
    auth::account_token,
    context::get_extension_context,
    request::{
        stream::{make_stream_request, StreamResponseState},
        JsonSendable,
    },
};

const SIGN_IN_ITEM: &str = "Sign In / Sign Up";
const CONFIGURE_API_KEY_ITEM: &str = "Configure API Key";

pub async fn make_stream<T>(path: &str, body: &T) -> Result<StreamResponseState, JsValue>
where
    T: Serialize,
{
    let mut request =
        make_stream_request(path, body).add_header("content-type", "application/connect+json");

    if let Some(token) = account_token() {
        request = request.add_header("Authorization", &format!("Bearer {}", token.access_token));
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

#[derive(Debug, Clone)]
pub struct FlaggedData {
    pub data: Vec<u8>,
    pub flag: u8,
}

impl FlaggedData {
    pub fn new<T>(data: T, flag: u8) -> Self
    where
        T: AsRef<[u8]>,
    {
        Self {
            data: data.as_ref().to_vec(),
            flag,
        }
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut result = vec![self.flag];
        result.extend_from_slice(&(self.data.len() as u32).to_be_bytes());
        result.extend_from_slice(&self.data);
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode() {
        let bytes = [1u8, 2, 3, 4];
        let result = FlaggedData::new(&bytes, 1).encode();
        assert_eq!(result, vec![1, 0, 0, 0, 4, 1, 2, 3, 4]);
    }
}
