use std::future::IntoFuture;

use crate::{auth::random_bytes, request::make_request};

use super::token::Token;

#[wasm_bindgen(getter_with_clone, js_name = AuthSession)]
pub struct Session {
    #[wasm_bindgen(js_name = loginUrl)]
    pub login_url: String,

    uuid: String,
    verifier: String,
}

#[wasm_bindgen(js_class = AuthSession)]
impl Session {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            login_url: format!(
                "https://cursor.so/loginDeepControl?challenge={challenge}&uuid={}",
                uuid.clone()
            )
            .to_owned(),
            uuid,
            verifier,
        }
    }

    pub async fn polling(&self, abort_signal: AbortSignal) -> Result<Option<String>, JsValue> {}
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
