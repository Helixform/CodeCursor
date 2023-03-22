use wasm_bindgen::prelude::*;

#[wasm_bindgen(module = "uuid")]
extern "C" {
    #[wasm_bindgen(js_name = "v4")]
    pub fn uuid_v4() -> String;
}
