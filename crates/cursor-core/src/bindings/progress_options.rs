use super::progress_location::ProgressLocation;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(getter_with_clone, js_name = RustProgressOptions)]
pub struct ProgressOptions {
    pub location: ProgressLocation,
    pub title: Option<String>,
    pub cancellable: bool,
}
