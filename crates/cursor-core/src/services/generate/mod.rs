pub mod models;

use futures::StreamExt;
use node_bridge::prelude::console;
use wasm_bindgen::JsValue;

use crate::GenerateInput;

use self::models::request_body::RequestBody;

use super::{flagged_chunk::FlaggedChunk, stream::make_stream};

#[derive(Debug, Clone, Copy)]
pub enum CodeGenerateMode {
    Generate,
    Edit,
}

#[derive(Debug, Clone)]
pub struct CodeGenerateService {
    mode: CodeGenerateMode,
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
            let data = chunk.utf8_string()?;
        }
        drop(data_stream);

        // Make sure the response is fully received without errors.
        state.complete().await?;
        result_stream.end();

        Ok(())
    }
}
