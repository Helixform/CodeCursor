use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    pub type AbortSignal;

    #[wasm_bindgen(method, getter)]
    pub fn aborted(this: &AbortSignal) -> bool;

    #[wasm_bindgen(method, js_name = addEventListener)]
    pub fn add_event_listener(this: &AbortSignal, event: &str, listener: JsValue);
}
