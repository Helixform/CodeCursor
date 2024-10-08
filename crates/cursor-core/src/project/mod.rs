mod handler;

use std::future::IntoFuture;

use futures::{
    future::{select, Either},
    StreamExt,
};
use node_bridge::{bindings::AbortSignal, futures::Defer, http_client::HttpMethod, prelude::*};
use serde_json::json;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::future_to_promise;

use crate::{
    bindings::{
        progress::Progress, progress_location::ProgressLocation, progress_options::ProgressOptions,
    },
    context::get_extension_context,
    request::{make_request, stream::StreamResponseState, JsonSendable, INTERNAL_HOST},
};

use self::handler::ProjectHandler;

const STEP_MESSAGE: &str = "cursor-step";
const CREATE_MESSAGE: &str = "cursor-create";
const APPEND_MESSAGE: &str = "cursor-append";
const END_MESSAGE: &str = "cursor-end";
const FINISHED_MESSAGE: &str = "cursor-finished";

enum Task {
    Step(String),
    Create(String),
    Append(String),
}

impl Task {
    fn title(&self) -> &str {
        match self {
            Task::Step(title) => title,
            Task::Create(title) => title,
            Task::Append(title) => title,
        }
    }
}

#[wasm_bindgen(js_name = generateProject)]
pub async fn generate_project(prompt: &str, handler: ProjectHandler) -> Result<JsValue, JsValue> {
    let prompt = prompt.to_owned();
    Ok(get_extension_context()
        .with_progress(
            ProgressOptions {
                location: ProgressLocation::Notification,
                title: Some("Generating project...".to_owned()),
                cancellable: true,
            },
            closure_once!(|progress: Progress, abort_signal: AbortSignal| {
                let defer_abort = Defer::new();
                let defer_abort_clone = defer_abort.clone();
                abort_signal.add_event_listener(
                    "abort",
                    closure_once!(|| {
                        defer_abort_clone.resolve(JsValue::null());
                    })
                    .into_js_value(),
                );

                let task = async move {
                    let mut state: StreamResponseState =
                        make_request(INTERNAL_HOST, "/gen_project", HttpMethod::Post)
                            .set_json_body(&json!({ "description": prompt }))
                            .send()
                            .await?
                            .into();
                    let mut data_stream = state.data_stream();
                    let mut current_task = None;
                    let mut file_writer = None;
                    while let Some(chunk) = data_stream.next().await {
                        // Each chunk contains multiple lines of data, which need to be separated and processed individually.
                        for data in String::from_utf8(chunk)
                            .map_err(|e| e.to_string())?
                            .split('\n')
                            .filter_map(|line| {
                                if !line.is_empty() && line.starts_with("data: \"") {
                                    serde_json::from_str::<String>(&line["data: ".len()..]).ok()
                                } else {
                                    None
                                }
                            })
                            .filter(|s| s != "[DONE]")
                        {
                            #[cfg(debug_assertions)]
                            console::log_str(&data);

                            // The start identifier of the task is in the form of: `identifier task`.
                            // First, match the prefix of the identifier,
                            // and then extract the specific task following it.
                            if data.starts_with(STEP_MESSAGE) {
                                let task = data[STEP_MESSAGE.len() + 1..].trim();
                                current_task = Some(Task::Step(task.to_owned()));
                            } else if data.starts_with(CREATE_MESSAGE) {
                                let task = data[CREATE_MESSAGE.len() + 1..].trim();
                                current_task = Some(Task::Create(format!("Creating {task}")));

                                // The title of the "create" message is a file path,
                                // which requires creating a file based on the path.
                                handler.create_file_recursive(task).await;
                            } else if data.starts_with(APPEND_MESSAGE) {
                                let task = data[APPEND_MESSAGE.len() + 1..].trim();
                                current_task =
                                    Some(Task::Append(format!("Appending contents to {task}")));

                                file_writer = handler.make_file_writer(task);
                            } else if data.starts_with(END_MESSAGE) {
                                current_task = None;
                                if let Some(w) = file_writer.as_ref() {
                                    w.end()
                                }
                                file_writer = None;
                            } else if data.starts_with(FINISHED_MESSAGE) {
                                if let Some(w) = file_writer.as_ref() {
                                    w.end()
                                }
                                break;
                            } else if let Some(Task::Append(_)) = &current_task {
                                if let Some(writer) = file_writer.as_ref() {
                                    writer.write(&data);
                                }
                            }

                            // The message sent by the report will automatically disappear after a short period of time.
                            // In order to keep the text displayed on the dialog box, report the title every time data is returned.
                            if let Some(task) = &current_task {
                                progress.report(task.title());
                            }
                        }
                    }
                    drop(data_stream);
                    state.complete().await.map(|_| JsValue::null())
                };

                future_to_promise(async move {
                    let task = std::pin::pin!(task);
                    match select(defer_abort.into_future(), task).await {
                        Either::Left((_, _)) => Ok(JsValue::null()),
                        Either::Right((result, _)) => result,
                    }
                })
            })
            .into_js_value()
            .into(),
        )
        .await)
}
