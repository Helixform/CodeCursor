use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    // `console.log` family
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    pub fn log_str(s: &str);

    #[wasm_bindgen(js_namespace = console, js_name = log)]
    pub fn log1(arg1: &JsValue);

    #[wasm_bindgen(js_namespace = console, js_name = log)]
    pub fn log2(arg1: &JsValue, arg2: &JsValue);

    #[wasm_bindgen(js_namespace = console, js_name = log)]
    pub fn log3(arg1: &JsValue, arg2: &JsValue, arg3: &JsValue);

    #[wasm_bindgen(js_namespace = console, js_name = log)]
    pub fn log4(arg1: &JsValue, arg2: &JsValue, arg3: &JsValue, arg4: &JsValue);

    // `console.warn` family
    #[wasm_bindgen(js_namespace = console, js_name = warn)]
    pub fn warn_str(s: &str);

    #[wasm_bindgen(js_namespace = console, js_name = warn)]
    pub fn warn1(arg1: &JsValue);

    #[wasm_bindgen(js_namespace = console, js_name = warn)]
    pub fn warn2(arg1: &JsValue, arg2: &JsValue);

    #[wasm_bindgen(js_namespace = console, js_name = warn)]
    pub fn warn3(arg1: &JsValue, arg2: &JsValue, arg3: &JsValue);

    #[wasm_bindgen(js_namespace = console, js_name = warn)]
    pub fn warn4(arg1: &JsValue, arg2: &JsValue, arg3: &JsValue, arg4: &JsValue);

    // `console.error` family
    #[wasm_bindgen(js_namespace = console, js_name = error)]
    pub fn error_str(s: &str);

    #[wasm_bindgen(js_namespace = console, js_name = error)]
    pub fn error1(arg1: &JsValue);

    #[wasm_bindgen(js_namespace = console, js_name = error)]
    pub fn error2(arg1: &JsValue, arg2: &JsValue);

    #[wasm_bindgen(js_namespace = console, js_name = error)]
    pub fn error3(arg1: &JsValue, arg2: &JsValue, arg3: &JsValue);

    #[wasm_bindgen(js_namespace = console, js_name = error)]
    pub fn error4(arg1: &JsValue, arg2: &JsValue, arg3: &JsValue, arg4: &JsValue);
}
