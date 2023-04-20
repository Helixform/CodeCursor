use wasm_bindgen::prelude::*;

#[wasm_bindgen(typescript_custom_section)]
const IPROGRESS: &'static str = r#"
interface IProgress {
    report(message?: string): void;
}
"#;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "IProgress")]
    pub type Progress;

    #[wasm_bindgen(method, structural)]
    pub fn report(this: &Progress, message: &str);
}
