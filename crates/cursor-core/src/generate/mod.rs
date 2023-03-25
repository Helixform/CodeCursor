use std::future::IntoFuture;

use crate::{models::*, request::make_request, GenerateInput};
use futures::{
    future::{select, Either},
    StreamExt,
};
use node_bridge::futures::Defer;
use node_bridge::prelude::*;
use wasm_bindgen::prelude::*;

async fn generate_code_inner(input: &GenerateInput) -> Result<(), JsValue> {
    let file_path = input.file_path();
    let selection = input.selection_range();

    let message_type = if selection.is_empty() {
        MessageType::Generate
    } else {
        MessageType::Edit
    };
    let mut request_body = RequestBody::new_with_input(input, message_type);

    let result_stream = input.result_stream();

    let mut conversation_id: Option<String> = None;
    // The last message received from the server.
    let mut previous_message: String = "".to_owned();
    let mut last_token = "".to_owned();

    // If the conversation was interrupted, we need to send a "continue" request.
    let mut interrupted = false;

    loop {
        #[cfg(debug_assertions)]
        console::log_str(&serde_json::to_string(&request_body).unwrap());

        let mut state = make_request(
            if interrupted {
                "/continue/"
            } else {
                "/conversation"
            },
            &request_body,
            !interrupted,
        )
        .await?;
        interrupted = false;

        #[cfg(debug_assertions)]
        console::log_str("response received");

        let mut data_stream = state.data_stream();
        while let Some(mut data) = data_stream.next().await {
            if data.contains("<|END_interrupt|>") {
                interrupted = true;
                last_token = data.clone();
                // `END_interrupt` is included in valid data,
                // we cannot discard it.
                data = data.replace("<|END_interrupt|>", "");
            }
            previous_message.push_str(&data);
            result_stream.write(&data);

            #[cfg(debug_assertions)]
            console::log_str(&format!("wrote: {}", &data));
        }
        drop(data_stream);

        // Make sure the response is fully received without errors.
        state.complete().await?;

        if !interrupted {
            break;
        }

        #[cfg(debug_assertions)]
        console::log_str("generation interrupted");

        // Generate an UUID as conversation ID.
        if conversation_id.is_none() {
            conversation_id = Some(node_bridge::bindings::uuid::uuid_v4());
        }
        let bot_message = BotMessage::new(
            conversation_id.clone().unwrap(),
            message_type,
            previous_message.clone(),
            last_token.clone(),
            file_path.to_owned(),
            false,
        );
        request_body.bot_messages = vec![bot_message];
    }

    console::log_str("generate done");

    result_stream.end();
    Ok(())
}

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
