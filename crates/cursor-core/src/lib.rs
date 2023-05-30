pub mod auth;
mod bindings;
pub mod context;
mod conversation;
mod project;
mod request;
pub mod services;
pub mod storage;

use node_bridge::bindings::AbortSignal;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(typescript_custom_section)]
const ISELECTION_RANGE: &'static str = r#"
interface ISelectionRange {
    get start(): IPosition;
    get end(): IPosition;
}
"#;

#[wasm_bindgen(typescript_custom_section)]
const IPOSITION: &'static str = r#"
interface IPosition {
    get line(): number;
    get character(): number;
}
"#;

#[wasm_bindgen(typescript_custom_section)]
const IRESULT_STREAM: &'static str = r#"
interface IResultStream {
    write(contents: string): void;
    end(): void;
}
"#;

#[wasm_bindgen(typescript_custom_section)]
const IGENERATE_INPUT: &'static str = r#"
interface IGenerateInput {
    get prompt(): string;
    get documentText(): string;
    get filePath(): string;
    get workspaceDirectory(): string | null;
    get cursor(): IPosition;
    get selectionRange(): ISelectionRange;
    get resultStream(): IResultStream;
    get abortSignal(): AbortSignal;
    get apiKey(): string | null;
    get gptModel(): string | null;
    get languageId(): string;
}
"#;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "ISelectionRange")]
    pub type SelectionRange;

    #[wasm_bindgen(method, getter, structural)]
    pub fn start(this: &SelectionRange) -> Position;

    #[wasm_bindgen(method, getter, structural)]
    pub fn end(this: &SelectionRange) -> Position;
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "IPosition")]
    pub type Position;

    #[wasm_bindgen(method, getter, structural)]
    pub fn line(this: &Position) -> usize;

    #[wasm_bindgen(method, getter, structural)]
    pub fn character(this: &Position) -> usize;
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "IResultStream")]
    pub type ResultStream;

    #[wasm_bindgen(method, structural)]
    pub fn write(this: &ResultStream, contents: &str);

    #[wasm_bindgen(method, structural)]
    pub fn end(this: &ResultStream);
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "IGenerateInput")]
    pub type GenerateInput;

    #[wasm_bindgen(method, getter, structural)]
    pub fn prompt(this: &GenerateInput) -> String;

    #[wasm_bindgen(method, getter, structural, js_name = documentText)]
    pub fn document_text(this: &GenerateInput) -> String;

    #[wasm_bindgen(method, getter, structural, js_name = filePath)]
    pub fn file_path(this: &GenerateInput) -> String;

    #[wasm_bindgen(method, getter, structural, js_name = workspaceDirectory)]
    pub fn workspace_directory(this: &GenerateInput) -> Option<String>;

    #[wasm_bindgen(method, getter, structural, js_name = selectionRange)]
    pub fn selection_range(this: &GenerateInput) -> SelectionRange;

    #[wasm_bindgen(method, getter, structural, js_name = cursor)]
    pub fn cursor(this: &GenerateInput) -> Position;

    #[wasm_bindgen(method, getter, structural, js_name = resultStream)]
    pub fn result_stream(this: &GenerateInput) -> ResultStream;

    #[wasm_bindgen(method, getter, structural, js_name = abortSignal)]
    pub fn abort_signal(this: &GenerateInput) -> AbortSignal;

    #[wasm_bindgen(method, getter, structural, js_name = apiKey)]
    pub fn api_key(this: &GenerateInput) -> Option<String>;

    #[wasm_bindgen(method, getter, structural, js_name = gptModel)]
    pub fn gpt_model(this: &GenerateInput) -> Option<String>;

    #[wasm_bindgen(method, getter, structural, js_name = languageId)]
    pub fn language_id(this: &GenerateInput) -> String;
}

impl GenerateInput {
    pub fn file_dir(&self) -> String {
        let file_path = self.file_path();
        return file_path
            .split("/")
            .take(file_path.split("/").count() - 1)
            .collect::<Vec<&str>>()
            .join("/");
    }
}
