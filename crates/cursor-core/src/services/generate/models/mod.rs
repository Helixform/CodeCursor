mod bot_message;
mod code_area;
pub(super) mod current_file;
pub(super) mod request_body;
pub(super) mod user_message;
pub(super) mod user_request;

pub(super) use bot_message::*;
pub(super) use request_body::*;
pub(super) use user_request::*;

// Split the code into chunks of 20 line blocks.
pub fn split_code_into_blocks(code: &str) -> Vec<String> {
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
