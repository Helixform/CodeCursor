use js_sys::Array;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    pub type Buffer;

    #[wasm_bindgen(static_method_of = Buffer, js_name = from)]
    pub fn from_str(data: &str, encoding: &str) -> Buffer;

    #[wasm_bindgen(static_method_of = Buffer, js_name = from)]
    pub fn from_array(data: Vec<u8>) -> Buffer;

    #[wasm_bindgen(method, js_name = toString)]
    pub fn to_string(this: &Buffer, encoding: &str) -> String;

    #[wasm_bindgen(static_method_of = Buffer)]
    pub fn concat(list: Array) -> Buffer;
}
