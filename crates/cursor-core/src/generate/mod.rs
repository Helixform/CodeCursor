mod models;

use crate::GenerateInput;
use futures::StreamExt;
use models::*;
use node_bridge::http_client::{HttpMethod, HttpRequest};
use wasm_bindgen::prelude::*;

// Split the code into chunks of 20 line blocks.
fn split_code_into_blocks(code: &str) -> Vec<String> {
    let lines = code.split("\n");
    let mut blocks = vec![];
    let mut current_block = vec![];
    for line in lines {
        current_block.push(line.to_string());
        if current_block.len() >= 20 {
            blocks.push(current_block.join("\n"));
            current_block = vec![];
        }
    }
    if current_block.len() > 0 {
        blocks.push(current_block.join("\n"));
    }
    blocks
}

#[wasm_bindgen(js_name = generateCode)]
pub async fn generate_code(input: &GenerateInput) -> Result<(), JsValue> {
    let file_path = input.file_path();
    let file_dir = file_path
        .split("/")
        .take(file_path.split("/").count() - 1)
        .collect::<Vec<&str>>()
        .join("/");
    node_bridge::bindings::console::log_str(&format!("file_dir: {}", file_dir));
    let workspace_directory = input.workspace_directory();
    let selection = input.selection_range();
    let document_text_utf16: Vec<u16> = input.document_text().encode_utf16().collect();

    let selection_text = if selection.length() > 0 {
        Some(String::from_utf16_lossy(
            &document_text_utf16[selection.offset()..selection.offset() + selection.length()],
        ))
    } else {
        None
    };
    let preceding_code = String::from_utf16_lossy(&document_text_utf16[0..selection.offset()]);
    let following_code =
        String::from_utf16_lossy(&document_text_utf16[selection.offset() + selection.length()..]);

    let message_type = if selection_text.is_some() {
        MessageType::Edit
    } else {
        MessageType::Generate
    };

    let prompt = input.prompt();

    let user_request = UserRequest::new(
        prompt,
        file_dir,
        file_path.to_owned(),
        input.document_text(),
        split_code_into_blocks(&preceding_code),
        split_code_into_blocks(&following_code),
        selection_text,
        message_type,
    );
    let mut request_body = RequestBody::new(user_request, vec![], workspace_directory);

    let result_stream = input.result_stream();

    // A Boolean value indicating whether the conversation is finished.
    let mut finished = false;
    // If the conversation was interrupted, we need to send a "continue" request.
    let mut interrupted = false;
    // Handle the SSE stream.
    let mut message_started = false;
    let mut first_newline_dropped = false;

    let mut conversation_id: Option<String> = None;
    // The last message received from the server.
    let mut previous_message: String = "".to_owned();
    let mut last_token = "".to_owned();

    while !finished {
        if interrupted {
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
            );
            request_body.bot_messages = vec![bot_message];
        }

        node_bridge::bindings::console::log_str(&serde_json::to_string(&request_body).unwrap());

        let request = HttpRequest::new(&format!(
            "https://aicursor.com/{}",
            if interrupted {
                "continue"
            } else {
                "conversation"
            }
        ))
        .set_method(HttpMethod::Post)
        .set_body(serde_json::to_string(&request_body).unwrap())
        .add_header("authority", "aicursor.com")
        .add_header("accept", "*/*")
        .add_header("content-type", "application/json")
        .add_header("user-agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Cursor/0.1.0 Chrome/108.0.5359.62 Electron/22.0.0 Safari/537.36");

        let mut response = request.send().await?;

        let body = response.body();
        while let Some(chunk) = body.next().await {
            let chunk = chunk.to_string("utf-8");
            node_bridge::bindings::console::log_str(&chunk);
            let lines = chunk.split("\n").filter(|l| l.len() > 0);
            let mut message_ended = false;
            for line in lines {
                if !line.starts_with("data: ") {
                    continue;
                }
                // A string can be JSON to parse.
                let data_str = &line["data: ".len()..];
                let mut data = serde_json::from_str::<String>(data_str).unwrap();
                if data == "<|BEGIN_message|>" {
                    message_started = true;
                    continue;
                } else if data.contains("<|END_interrupt|>") {
                    interrupted = true;
                    last_token = data.clone();
                    // `END_interrupt` is included in valid data,
                    // we cannot discard it.
                    data = data.replace("<|END_interrupt|>", "");
                } else if data == "<|END_message|>" {
                    if !interrupted {
                        finished = true;
                    }
                    // We cannot exit the loop here because we're in a nested loop.
                    message_ended = true;
                    break;
                }

                if message_started {
                    // Server may produce newlines at the head of response, we need
                    // to do this trick to ignore them in the final edit.
                    if !first_newline_dropped && data.trim().len() == 0 {
                        first_newline_dropped = true;
                        continue;
                    }
                    previous_message.push_str(&data);
                    result_stream.write(&data);
                }
            }
            // If we've reached the end of the message, break out of the loop.
            if message_ended {
                break;
            }
        }

        response.await?;
    }

    node_bridge::bindings::console::log_str("done");

    result_stream.end();
    Ok(())
}
