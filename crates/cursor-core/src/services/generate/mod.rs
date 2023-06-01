mod request_body;

use std::pin::pin;

use futures::StreamExt;
use node_bridge::prelude::console;
use serde::Deserialize;
use wasm_bindgen::{JsError, JsValue};

use crate::GenerateInput;

use self::request_body::RequestBody;

use super::stream::make_stream;

#[derive(Debug, Clone)]
pub struct CodeGenerateService;

#[derive(Debug, Clone, Deserialize)]
struct FilledPrompt {
    #[serde(rename = "filledPrompt")]
    pub text: String,
}

#[derive(Debug, Clone, Deserialize)]
struct ChunkContent {
    pub text: String,
}

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
                } else if let Ok(ChunkContent { text, .. }) =
                    serde_json::from_str::<ChunkContent>(&data)
                {
                    #[cfg(debug_assertions)]
                    console::log_str(&format!("wrote: {}", text));
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
