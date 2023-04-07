mod session;

use std::cell::RefCell;
use std::future::IntoFuture;

use futures::future::{select, Either};
use node_bridge::futures::Defer;
use node_bridge::prelude::*;
use wasm_bindgen::prelude::*;

use crate::GenerateInput;
use session::Session;

enum SharedSessionState {
    Available(Option<Session>),
    Occupied,
}

thread_local! {
    static SHARED_SESSION: RefCell<SharedSessionState> = RefCell::new(SharedSessionState::Available(None));
}

#[wasm_bindgen(js_name = resetChat)]
pub fn reset_chat() {
    SHARED_SESSION
        .with(|shared_session| shared_session.replace(SharedSessionState::Available(None)));
}

#[wasm_bindgen(js_name = chat)]
pub async fn chat(input: &GenerateInput) -> Result<(), JsValue> {
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

    // Get the shared session, create a new one if not existed.
    let mut session = SHARED_SESSION.with(|shared_session| {
        let session = match &mut *shared_session.borrow_mut() {
            SharedSessionState::Occupied => {
                // TODO: throw an error.
                unreachable!("Cannot invoke chat concurrently")
            }
            SharedSessionState::Available(session) => session.take().unwrap_or_default(),
        };
        shared_session.replace(SharedSessionState::Occupied);
        session
    });

    let fut = session.send_message(input);

    let result = match select(defer_abort.into_future(), Box::pin(fut)).await {
        Either::Left(_) => Ok(()),
        Either::Right((res, _)) => res,
    };

    // Put back the session to the global state, unlocking it for the next call.
    SHARED_SESSION.with(|shared_session| {
        shared_session.replace(SharedSessionState::Available(Some(session)))
    });

    result
}
