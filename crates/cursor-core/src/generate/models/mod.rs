pub(crate) mod bot_message;
pub(crate) mod request_body;
pub(crate) mod user_request;

pub(crate) use bot_message::*;
pub(crate) use request_body::*;
pub(crate) use user_request::*;

fn random() -> i32 {
    js_sys::Math::floor(js_sys::Math::random() * 1000.0) as i32
}
