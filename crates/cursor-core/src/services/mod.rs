mod flagged_chunk;
pub mod generate;
mod stream;

use std::future::IntoFuture;

use futures::future::{select, Either};
use node_bridge::{futures::Defer, prelude::*};
use wasm_bindgen::prelude::*;

use crate::GenerateInput;

use self::generate::{CodeGenerateMode, CodeGenerateService};

#[wasm_bindgen(js_name = generateCode)]
pub async fn generate_code(input: &GenerateInput) -> Result<(), JsValue> {
    let defer_abort = Defer::new();
    let defer_abort_clone = defer_abort.clone();
    let abort_signal = input.abort_signal();
    abort_signal.add_event_listener(
        "abort",
        closure_once!(|| {
            defer_abort_clone.resolve(JsValue::null());
        })
        .into_js_value(),
    );

    let service = CodeGenerateService::new(CodeGenerateMode::Generate);
    let fut = service.generate(input);

    let x = match select(defer_abort.into_future(), Box::pin(fut)).await {
        Either::Left(_) => Ok(()),
        Either::Right((res, _)) => res,
    };
    x
}
