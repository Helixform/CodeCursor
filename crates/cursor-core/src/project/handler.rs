use wasm_bindgen::prelude::*;

#[wasm_bindgen(typescript_custom_section)]
const IPROJECT_HANDLER: &'static str = r#"
interface IProjectHandler {
    createFileRecursive(path: string): Promise<void>;
    makeFileWriter(path: string): IProjectFileWriter | undefined;
}
"#;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "IProjectHandler")]
    pub type ProjectHandler;

    #[wasm_bindgen(method, structural, js_name = createFileRecursive)]
    pub async fn create_file_recursive(this: &ProjectHandler, path: &str);

    #[wasm_bindgen(method, structural, js_name = makeFileWriter)]
    pub fn make_file_writer(this: &ProjectHandler, path: &str) -> Option<ProjectFileWriter>;
}

#[wasm_bindgen(typescript_custom_section)]
const IPROJECT_FILE_WRITER: &'static str = r#"
interface IProjectFileWriter {
    write(contents: string): void;
    end(): void;
}
"#;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "IProjectFileWriter")]
    pub type ProjectFileWriter;

    #[wasm_bindgen(method, structural)]
    pub fn write(this: &ProjectFileWriter, contents: &str);

    #[wasm_bindgen(method, structural)]
    pub fn end(this: &ProjectFileWriter);
}
