use std::collections::HashMap;
use std::fmt::Debug;
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context, Poll};

use js_sys::{Object as JsObject, Reflect};
use wasm_bindgen::prelude::*;

use crate::bindings::{https::*, Buffer};
use crate::futures::Defer;

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

#[derive(Clone)]
struct HttpResponseDataHandler(Rc<Closure<dyn FnMut(Buffer)>>);

impl Debug for HttpResponseDataHandler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("HttpResponseDataHandler")
            .field(&self.0)
            .finish()
    }
}

#[derive(Clone, Debug)]
pub struct HttpRequest {
    url: String,
    method: HttpMethod,
    headers: HashMap<String, String>,
    body: Option<String>,
    data_handler: Option<HttpResponseDataHandler>,
}

impl HttpRequest {
    pub fn new(url: &str) -> Self {
        Self {
            url: url.to_owned(),
            method: HttpMethod::Get,
            headers: HashMap::new(),
            body: None,

            data_handler: None,
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

    pub fn set_data_handler<F>(mut self, handler: F) -> Self
    where
        F: FnMut(Buffer) + 'static,
    {
        self.data_handler = Some(HttpResponseDataHandler(Rc::new(Closure::new(handler))));
        self
    }

    pub fn send(self) -> Result<SentHttpRequest, JsValue> {
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

        Ok(SentHttpRequest::new(req, self.data_handler))
    }
}

pub struct SentHttpRequest {
    fut: Pin<Box<dyn Future<Output = Result<(), JsValue>>>>,
}

impl SentHttpRequest {
    fn new(req: ClientRequest, data_handler: Option<HttpResponseDataHandler>) -> Self {
        let fut = async move {
            // Wait for the response.
            let defer_resp = Defer::new();
            let defer_resp_clone = defer_resp.clone();
            let on_resp_closure: Closure<dyn FnMut(_)> =
                Closure::new(move |resp: IncomingMessage| {
                    defer_resp_clone.resolve(resp.into());
                });
            req.on("response", on_resp_closure.as_ref());
            let resp: IncomingMessage = defer_resp.await?.into();

            // Receive data from the response body and wait until it ends.
            let data_handler = data_handler;
            if let Some(data_handler) = &data_handler {
                resp.on("data", (*data_handler.0).as_ref());
            }

            let defer_end = Defer::new();
            let defer_end_clone = defer_end.clone();
            let on_end_closure: Closure<dyn FnMut()> = Closure::new(move || {
                defer_end_clone.resolve(JsValue::UNDEFINED);
            });
            resp.on("end", on_end_closure.as_ref());
            defer_end.await?;

            Ok(())
        };

        Self { fut: Box::pin(fut) }
    }
}

impl Future for SentHttpRequest {
    type Output = Result<(), JsValue>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let fut = self.fut.as_mut();
        fut.poll(cx)
    }
}
