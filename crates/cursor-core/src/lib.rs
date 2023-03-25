mod chat;
mod generate;
mod models;
mod request;

use node_bridge::bindings::AbortSignal;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(typescript_custom_section)]
const ISELECTION_RANGE: &'static str = r#"
interface ISelectionRange {
    get offset(): number;
    get length(): number;
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
    get selectionRange(): ISelectionRange;
    get resultStream(): IResultStream;
    get abortSignal(): AbortSignal;
}
"#;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "ISelectionRange")]
    pub type SelectionRange;

    #[wasm_bindgen(method, getter, structural)]
    pub fn offset(this: &SelectionRange) -> usize;

    #[wasm_bindgen(method, getter, structural)]
    pub fn length(this: &SelectionRange) -> usize;
}

impl SelectionRange {
    pub fn is_empty(&self) -> bool {
        self.length() == 0
    }
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

    #[wasm_bindgen(method, getter, structural, js_name = resultStream)]
    pub fn result_stream(this: &GenerateInput) -> ResultStream;

    #[wasm_bindgen(method, getter, structural, js_name = abortSignal)]
    pub fn abort_signal(this: &GenerateInput) -> AbortSignal;
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
