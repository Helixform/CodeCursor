use std::future::IntoFuture;

use futures::{
    future::{select, Either},
    StreamExt,
};
use node_bridge::futures::Defer;
use node_bridge::prelude::*;
use wasm_bindgen::prelude::*;

use crate::GenerateInput;

use super::{
    models::{MessageType, RequestBody},
    send_conversation_request,
};

async fn generate_code_inner(input: &GenerateInput) -> Result<(), JsValue> {
    let selection = input.selection_range();

    let message_type = MessageType::Generate;
    let request_body = RequestBody::new_with_input(input, message_type);
    let result_stream = input.result_stream();

    #[cfg(debug_assertions)]
    console::log_str(&serde_json::to_string(&request_body).unwrap());

    let mut state = send_conversation_request("/conversation", &request_body).await?;

    #[cfg(debug_assertions)]
    console::log_str("response received");

    let mut data_stream = state.data_stream();
    while let Some(data) = data_stream.next().await {
        result_stream.write(&data);
        #[cfg(debug_assertions)]
        console::log_str(&format!("wrote: {}", &data));
    }
    drop(data_stream);

    // Make sure the response is fully received without errors.
    state.complete().await?;
    result_stream.end();
    Ok(())
}

#[wasm_bindgen(js_name = generateCode)]
pub async fn generate_code(input: &GenerateInput) -> Result<(), JsValue> {
    console::log1(input);
    return Ok(());
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

    let fut = generate_code_inner(input);

    match select(defer_abort.into_future(), Box::pin(fut)).await {
        Either::Left(_) => {
            return Ok(());
        }
        Either::Right((res, _)) => {
            return res;
        }
    }
}
