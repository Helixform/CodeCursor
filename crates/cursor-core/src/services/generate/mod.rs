pub mod models;

use futures::StreamExt;
use node_bridge::prelude::console;
use serde::Deserialize;
use wasm_bindgen::{JsError, JsValue};

use crate::GenerateInput;

use self::models::request_body::RequestBody;

use super::stream::make_stream;

#[derive(Debug, Clone, Copy)]
pub enum CodeGenerateMode {
    Generate,
    Edit,
}

#[derive(Debug, Clone)]
pub struct CodeGenerateService {
    mode: CodeGenerateMode,
}

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
    pub fn new(mode: CodeGenerateMode) -> Self {
        Self { mode }
    }

    pub async fn generate(&self, input: &GenerateInput) -> Result<(), JsValue> {
        let mut state = make_stream(
            "/aiserver.v1.AiService/StreamGenerate",
            &RequestBody::new_with_input(input),
        )
        .await?;

        let result_stream = input.result_stream();
        let mut data_stream = state.data_stream();
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
        drop(data_stream);

        // Make sure the response is fully received without errors.
        state.complete().await?;
        result_stream.end();

        Ok(())
    }
}
