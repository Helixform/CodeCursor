use std::{cell::RefCell, ops::Deref};

use crate::{bindings::progress_options::ProgressOptions, storage::GlobalStorage};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(typescript_custom_section)]
const IEXTENSION_CONTEXT: &'static str = r#"
interface IExtensionContext {
    get storage(): IGlobalStorage;
    executeCommand(command: string, ...args: any[]): Thenable<any>;
    withProgress(options: RustProgressOptions, callback: () => Thenable<any>): Thenable<any>;
}
"#;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "IExtensionContext")]
    pub type ExtensionContext;

    #[wasm_bindgen(method, structural, getter)]
    pub fn storage(this: &ExtensionContext) -> GlobalStorage;

    #[wasm_bindgen(method, structural, js_name = executeCommand)]
    pub async fn execute_command1(this: &ExtensionContext, command: &str, args: JsValue)
        -> JsValue;

    #[wasm_bindgen(method, structural, js_name = withProgress)]
    pub async fn with_progress(
        this: &ExtensionContext,
        options: ProgressOptions,
        callback: js_sys::Function,
    ) -> JsValue;
}

thread_local! {
    static EXTENSION_CONTEXT: RefCell<Option<ExtensionContext>> = RefCell::new(None);
}

#[wasm_bindgen(js_name = setExtensionContext)]
pub fn set_extension_context(context: ExtensionContext) {
    EXTENSION_CONTEXT.with(|ctx| ctx.replace(Some(context)));
}

pub fn get_extension_context() -> ExtensionContext {
    EXTENSION_CONTEXT.with(|ctx| {
        return ctx.borrow().as_ref().unwrap().deref().clone().into();
    })
}
