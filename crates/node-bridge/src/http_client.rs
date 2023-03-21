use std::collections::HashMap;
use std::fmt::Debug;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

use js_sys::{Object as JsObject, Reflect};
use wasm_bindgen::prelude::*;

use crate::bindings::{https::*, Buffer};
use crate::futures::{AsyncIter, Defer};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
}

impl ToString for HttpMethod {
    fn to_string(&self) -> String {
        match self {
            HttpMethod::Get => "GET".to_owned(),
            HttpMethod::Post => "POST".to_owned(),
            HttpMethod::Put => "PUT".to_owned(),
            HttpMethod::Delete => "DELETE".to_owned(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct HttpRequest {
    url: String,
    method: HttpMethod,
    headers: HashMap<String, String>,
    body: Option<String>,
}

impl HttpRequest {
    pub fn new(url: &str) -> Self {
        Self {
            url: url.to_owned(),
            method: HttpMethod::Get,
            headers: HashMap::new(),
            body: None,
        }
    }

    pub fn set_method(mut self, method: HttpMethod) -> Self {
        self.method = method;
        self
    }

    pub fn add_header(mut self, header_field: &str, value: &str) -> Self {
        self.headers
            .insert(header_field.to_owned(), value.to_owned());
        self
    }

    pub fn set_body(mut self, body: String) -> Self {
        self.body = Some(body);
        self
    }

    pub async fn send(self) -> Result<HttpResponse, JsValue> {
        // Setup the request.
        let options = JsObject::new();
        Reflect::set(&options, &"method".into(), &self.method.to_string().into())?;

        let headers_obj = JsObject::new();
        for (header_field, value) in &self.headers {
            Reflect::set(&headers_obj, &header_field.into(), &value.into())?;
        }
        Reflect::set(&options, &"headers".into(), &headers_obj)?;

        let req = request(&self.url, &options);

        // Send the request with an optional body.
        if let Some(body) = self.body {
            let body_buf = Buffer::from_str(&body, "utf-8");
            req.write(body_buf);
        }
        req.end();

        // Wait for the response.
        let defer_resp = Defer::new();
        let defer_resp_clone = defer_resp.clone();
        let on_resp_closure: Closure<dyn FnMut(_)> = Closure::new(move |resp: IncomingMessage| {
            defer_resp_clone.resolve(resp.into());
        });
        req.on("response", on_resp_closure.as_ref());
        let resp: IncomingMessage = defer_resp.await?.into();

        Ok(HttpResponse::new(resp))
    }
}

pub struct HttpResponse {
    fut: Pin<Box<dyn Future<Output = Result<(), JsValue>>>>,
    data_stream: AsyncIter<Buffer>,

    // Ensure the closure alive during receiving response data.
    #[allow(dead_code)]
    on_data_closure: Closure<dyn FnMut(Buffer)>,
}

impl HttpResponse {
    fn new(resp: IncomingMessage) -> Self {
        let data_stream = AsyncIter::new();
        let mut data_stream_sender = data_stream.sender();

        let on_data_closure: Closure<dyn FnMut(_)> = Closure::new(move |chunk: Buffer| {
            data_stream_sender.send(chunk);
        });
        resp.on("data", on_data_closure.as_ref());

        let fut = async move {
            let defer_close = Defer::new();
            let defer_close_clone = defer_close.clone();
            let on_close_closure: Closure<dyn FnMut()> = Closure::new(move || {
                defer_close_clone.resolve(JsValue::UNDEFINED);
            });
            resp.on("close", on_close_closure.as_ref());
            defer_close.await?;

            Ok(())
        };

        Self {
            fut: Box::pin(fut),
            data_stream,
            on_data_closure,
        }
    }

    pub fn body(&mut self) -> &mut AsyncIter<Buffer> {
        &mut self.data_stream
    }
}

impl Future for HttpResponse {
    type Output = Result<(), JsValue>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let fut = self.fut.as_mut();
        fut.poll(cx)
    }
}
