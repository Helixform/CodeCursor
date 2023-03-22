use std::collections::HashMap;
use std::fmt::Debug;
use std::future::{Future, IntoFuture};
use std::pin::Pin;
use std::task::{Context, Poll};

use futures::future::{select, Either};
use js_sys::{Object as JsObject, Reflect};
use wasm_bindgen::prelude::*;

use crate::bindings::{https::*, Buffer};
use crate::futures::{AsyncIter, Defer};
use crate::{closure, closure_once};

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
            HttpMethod::Get => "GET",
            HttpMethod::Post => "POST",
            HttpMethod::Put => "PUT",
            HttpMethod::Delete => "DELETE",
        }
        .to_owned()
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

        let req = request(&self.url, options.into());

        let defer_resp = Defer::new();
        let defer_resp_clone = defer_resp.clone();
        req.on(
            "response",
            closure!(|resp: IncomingMessage| {
                defer_resp_clone.resolve(resp.into());
            })
            .into_js_value(),
        );

        let defer_err = Defer::new();
        let defer_err_clone = defer_err.clone();
        req.on(
            "error",
            closure_once!(|err: JsValue| {
                crate::bindings::console::error1(&err);
                defer_err_clone.resolve(err);
            })
            .into_js_value(),
        );

        // Send the request with an optional body.
        if let Some(body) = self.body {
            let body_buf = Buffer::from_str(&body, "utf-8");
            req.write(body_buf);
        }
        req.end();

        #[cfg(debug_assertions)]
        crate::bindings::console::log2(&"request sent: ".into(), &req);

        // Wait for the response.
        let resp: IncomingMessage =
            match select(defer_resp.into_future(), defer_err.into_future()).await {
                Either::Left((Ok(resp), _)) => Ok(resp),
                Either::Right((Ok(err), _)) => Err(err),
                _ => unreachable!("Impossible code path"),
            }?
            .into();

        #[cfg(debug_assertions)]
        crate::bindings::console::log2(&"response received: ".into(), &resp);

        Ok(HttpResponse::new(resp))
    }
}

pub struct HttpResponse {
    fut: Pin<Box<dyn Future<Output = Result<(), JsValue>>>>,
    data_stream: AsyncIter<Buffer>,
}

impl HttpResponse {
    fn new(resp: IncomingMessage) -> Self {
        let data_stream = AsyncIter::new();
        let mut data_stream_sender = data_stream.sender();
        let mut data_stream_sender_for_close = data_stream.sender();

        resp.on(
            "data",
            closure!(|chunk: Buffer| {
                #[cfg(debug_assertions)]
                crate::bindings::console::log_str("chunk received");
                data_stream_sender.send(Some(chunk));
            })
            .into_js_value(),
        );

        let defer_close = Defer::new();
        let defer_close_clone = defer_close.clone();
        resp.on(
            "close",
            closure_once!(|| {
                #[cfg(debug_assertions)]
                crate::bindings::console::log_str("response closed");
                data_stream_sender_for_close.send(None);
                defer_close_clone.resolve(JsValue::UNDEFINED);
            })
            .into_js_value(),
        );

        let fut = async move {
            defer_close.await?;
            Ok(())
        };

        Self {
            fut: Box::pin(fut),
            data_stream,
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
