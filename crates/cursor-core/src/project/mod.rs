mod handler;

use futures::StreamExt;
use node_bridge::{bindings::AbortSignal, prelude::*};
use serde_json::json;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::future_to_promise;

use crate::{
    bindings::{
        progress::Progress, progress_location::ProgressLocation, progress_options::ProgressOptions,
    },
    context::get_extension_context,
    request::stream::{make_stream_request, StreamResponseState},
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
                future_to_promise(async move {
                    let mut state: StreamResponseState =
                        make_stream_request("/gen_project", &json!({ "description": prompt }))
                            .send()
                            .await?
                            .into();
                    let mut data_stream = state.data_stream();
                    let mut current_task = None;
                    let mut file_writer = None;
                    while let Some(data) = data_stream.next().await {
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
                            current_task = Some(Task::Create(format!("Creating {}", task)));

                            // The title of the "create" message is a file path,
                            // which requires creating a file based on the path.
                            handler.create_file_recursive(task).await;
                        } else if data.starts_with(APPEND_MESSAGE) {
                            let task = data[APPEND_MESSAGE.len() + 1..].trim();
                            current_task =
                                Some(Task::Append(format!("Appending contents to {}", task)));

                            file_writer = handler.make_file_writer(task);
                        } else if data.starts_with(END_MESSAGE) {
                            current_task = None;
                            file_writer.as_ref().map(|w| w.end());
                            file_writer = None;
                        } else if data.starts_with(FINISHED_MESSAGE) {
                            file_writer.as_ref().map(|w| w.end());
                            break;
                        } else {
                            match &current_task {
                                Some(Task::Append(_)) => {
                                    if let Some(writer) = file_writer.as_ref() {
                                        writer.write(&data);
                                    }
                                }
                                _ => {}
                            }
                        }

                        // The message sent by the report will automatically disappear after a short period of time.
                        // In order to keep the text displayed on the dialog box, report the title every time data is returned.
                        if current_task.is_some() {
                            progress.report(current_task.as_ref().unwrap().title());
                        }
                    }
                    drop(data_stream);
                    state.complete().await.map(|_| JsValue::null())
                })
            })
            .into_js_value()
            .into(),
        )
        .await)
}
