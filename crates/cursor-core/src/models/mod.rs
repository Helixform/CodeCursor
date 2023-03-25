pub(crate) mod bot_message;
mod code_area;
pub(crate) mod request_body;
pub(crate) mod user_message;
pub(crate) mod user_request;

pub(crate) use bot_message::*;
pub(crate) use request_body::*;
pub(crate) use user_message::*;
pub(crate) use user_request::*;

fn random() -> i32 {
    js_sys::Math::floor(js_sys::Math::random() * 1000.0) as i32
}

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
