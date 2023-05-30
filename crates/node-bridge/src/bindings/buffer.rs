use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    pub type Buffer;

    #[wasm_bindgen(static_method_of = Buffer, js_name = from)]
    pub fn from_bytes(bytes: &[u8]) -> Buffer;

    #[wasm_bindgen(method, js_name = toString)]
    pub fn to_string(this: &Buffer, encoding: &str) -> String;
}
