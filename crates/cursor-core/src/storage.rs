use wasm_bindgen::prelude::*;

#[wasm_bindgen(typescript_custom_section)]
const IGLOBAL_STORAGE: &'static str = r#"
interface IGlobalStorage {
    update(key: string, value: string | null): void;
    get(key: string): string | null;
}
"#;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "IGlobalStorage")]
    pub type GlobalStorage;

    #[wasm_bindgen(method, structural)]
    pub fn update(this: &GlobalStorage, key: &str, value: Option<&str>);

    #[wasm_bindgen(method, structural)]
    pub fn get(this: &GlobalStorage, key: &str) -> Option<String>;
}
