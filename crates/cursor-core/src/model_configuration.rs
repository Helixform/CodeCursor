use wasm_bindgen::prelude::*;

#[wasm_bindgen(typescript_custom_section)]
const IMODEL_CONFIGURATION: &'static str = r#"
interface IModelConfiguration {
    get apiKey(): string | null;
    get gptModel(): string;
}
"#;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "IModelConfiguration")]
    pub type ModelConfiguration;

    #[wasm_bindgen(method, getter, structural, js_name = apiKey)]
    pub fn api_key(this: &ModelConfiguration) -> Option<String>;

    #[wasm_bindgen(method, getter, structural, js_name = gptModel)]
    pub fn model_name(this: &ModelConfiguration) -> String;
}
