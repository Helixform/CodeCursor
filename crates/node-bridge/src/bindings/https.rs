use wasm_bindgen::prelude::*;

use super::Buffer;

#[wasm_bindgen(module = "node:https")]
extern "C" {
    pub type ClientRequest;

    #[wasm_bindgen]
    pub fn request(url: &str, options: JsValue) -> ClientRequest;

    #[wasm_bindgen(method)]
    pub fn write(this: &ClientRequest, chunk: Buffer);

    #[wasm_bindgen(method)]
    pub fn end(this: &ClientRequest);

    #[wasm_bindgen(method)]
    pub fn on(this: &ClientRequest, event: &str, listener: JsValue);
}

#[wasm_bindgen(module = "node:https")]
extern "C" {
    pub type IncomingMessage;

    #[wasm_bindgen(method)]
    pub fn on(this: &IncomingMessage, event: &str, listener: JsValue);
}
