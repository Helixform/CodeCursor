use js_sys::Uint8Array;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    pub type TextEncoder;

    #[wasm_bindgen(constructor)]
    pub fn new() -> TextEncoder;

    #[wasm_bindgen(method, js_name = encode)]
    pub fn encode_to_uint8array(this: &TextEncoder, input: &str) -> Uint8Array;
}
