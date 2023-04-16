use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    pub type CancellationToken;

    #[wasm_bindgen(method, getter, structural, js_name = isCancellationRequested)]
    pub fn is_cancellation_requested(this: &CancellationToken) -> bool;
}
