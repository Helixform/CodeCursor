pub mod models;

use anyhow::Result;
use node_bridge::prelude::*;

use crate::GenerateInput;

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

    pub async fn generate(&self, input: &GenerateInput) -> Result<()> {
        // let selection = input.selection_range();

        // let message_type = if selection.is_empty() {
        //     MessageType::Generate
        // } else {
        //     MessageType::Edit
        // };
        // let request_body = RequestBody::new_with_input(input, message_type);
        // let result_stream = input.result_stream();

        // #[cfg(debug_assertions)]
        // console::log_str(&serde_json::to_string(&request_body).unwrap());

        // let mut state = send_conversation_request("/conversation", &request_body).await?;

        // #[cfg(debug_assertions)]
        // console::log_str("response received");

        // let mut data_stream = state.data_stream();
        // while let Some(data) = data_stream.next().await {
        //     result_stream.write(&data);
        //     #[cfg(debug_assertions)]
        //     console::log_str(&format!("wrote: {}", &data));
        // }
        // drop(data_stream);

        // // Make sure the response is fully received without errors.
        // state.complete().await?;
        // result_stream.end();
        Ok(())
    }
}
