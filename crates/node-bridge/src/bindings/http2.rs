use wasm_bindgen::prelude::*;

use super::Buffer;

#[wasm_bindgen(module = "node:http2")]
extern "C" {
    pub type ClientHttp2Session;
    pub type ClientHttp2Stream;
    pub type Http2Session;
   
   #[wasm_bindgen]
   pub fn connect(url: &str) -> ClientHttp2Session;

   #[wasm_bindgen(method)]
   pub fn request(this: &ClientHttp2Session, headers: JsValue) -> ClientHttp2Stream;

   #[wasm_bindgen(method)]
   pub fn close(this: &ClientHttp2Session);

   #[wasm_bindgen(method)]
   pub fn close(this: &Http2Session);

   #[wasm_bindgen(method)]
   pub fn write(this: &ClientHttp2Stream, chunk: Buffer);

   #[wasm_bindgen(method)]
   pub fn end(this: &ClientHttp2Stream);

   #[wasm_bindgen(method)]
   pub fn destroy(this: &ClientHttp2Stream);

   #[wasm_bindgen(method, getter)]
   pub fn session(this: &ClientHttp2Stream) -> Http2Session;

   #[wasm_bindgen(method)]
   pub fn on(this: &ClientHttp2Stream, event: &str, listener: JsValue);
}