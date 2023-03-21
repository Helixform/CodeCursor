use std::future::IntoFuture;

use js_sys::{Function, Promise};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

#[derive(Clone)]
pub struct Defer {
    resolver: (Function, Function),
    promise: Promise,
}

impl Defer {
    pub fn new() -> Self {
        let (mut resolve_f, mut reject_f) = (None, None);
        let promise = Promise::new(&mut |res, rej| {
            resolve_f = Some(res);
            reject_f = Some(rej);
        });

        let resolver = (resolve_f.unwrap(), reject_f.unwrap());

        Self {
            resolver,
            promise
        }
    }

    pub fn resolve(&self, value: JsValue) {
        self.resolver.0.call1(&JsValue::UNDEFINED, &value).unwrap();
    }

    pub fn reject(&self, error: JsValue) {
        self.resolver.1.call1(&JsValue::UNDEFINED, &error).unwrap();
    }
}

impl IntoFuture for Defer {
    type Output = Result<JsValue, JsValue>;

    type IntoFuture = JsFuture;

    fn into_future(self) -> Self::IntoFuture {
        JsFuture::from(self.promise)
    }
}
