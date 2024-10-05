mod request_body;

use std::{future::IntoFuture, pin::pin};

use futures::{
    future::{select, Either},
    StreamExt,
};
use node_bridge::{futures::Defer, prelude::*};
use wasm_bindgen::{prelude::*, JsError, JsValue};

use crate::GenerateInput;

use self::request_body::RequestBody;

use super::{
    enveloped_message::{FilledPrompt, MessageContent},
    stream::make_stream,
};

#[derive(Debug, Clone)]
pub struct CodeGenerateService;

impl CodeGenerateService {
    pub async fn generate(input: &GenerateInput) -> Result<(), JsValue> {
        let mut state = make_stream(
            &format!(
                "/aiserver.v1.AiService/Stream{}",
                if input.selection_range().is_empty() {
                    "Generate"
                } else {
                    "Edit"
                }
            ),
            &RequestBody::new_with_input(input),
        )
        .await?;

        let result_stream = input.result_stream();
        {
            let mut data_stream = pin!(state.data_stream());
            while let Some(chunk) = data_stream.next().await {
                if chunk.is_end() {
                    break;
                }
                let data = chunk
                    .utf8_string()
                    .map_err(|e| JsError::new(&e.to_string()))?;
                if data.is_empty() {
                    continue;
                }
                if let Ok(prompt) = serde_json::from_str::<FilledPrompt>(&data) {
                    #[cfg(debug_assertions)]
                    console::log_str(&format!("prompt: \n{}", prompt.text));
                    continue;
                } else if let Ok(MessageContent { text, .. }) =
                    serde_json::from_str::<MessageContent>(&data)
                {
                    #[cfg(debug_assertions)]
                    console::log_str(&format!("wrote: {text}"));
                    result_stream.write(&text);
                }
            }
        }

        // Make sure the response is fully received without errors.
        state.complete().await?;
        result_stream.end();

        Ok(())
    }
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

    let fut = CodeGenerateService::generate(input);

    match select(defer_abort.into_future(), Box::pin(fut)).await {
        Either::Left(_) => Ok(()),
        Either::Right((res, _)) => res,
    }
}
