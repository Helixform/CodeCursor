use std::collections::HashMap;
use std::fmt::Debug;
use std::future::{Future, IntoFuture};
use std::pin::Pin;
use std::task::{Context, Poll};

use futures::future::{select, Either};
use futures::StreamExt;
use js_sys::{Object as JsObject, Reflect};
use wasm_bindgen::prelude::*;

use crate::bindings::https::*;
use crate::futures::{AsyncIter, Defer};
use crate::prelude::*;
use crate::{closure, closure_once};

/// The Request Method (VERB)
///
/// Currently it does not cover all the methods defined in
/// [RFC 7230](https://tools.ietf.org/html/rfc7231#section-4.1),
/// which is fine for our usage.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HttpMethod {
    /// GET
    Get,
    /// POST
    Post,
    /// PUT
    Put,
    /// DELETE
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

/// An HTTP request.
///
/// When performing the request, it uses [`request`] from Node.js as
/// the underlying HTTP client.
#[derive(Clone, Debug)]
pub struct HttpRequest {
    url: String,
    method: HttpMethod,
    headers: HashMap<String, String>,
    body: Option<Vec<u8>>,
}

impl HttpRequest {
    /// Constructs a new request.
    pub fn new(url: &str) -> Self {
        Self {
            url: url.to_owned(),
            method: HttpMethod::Get,
            headers: HashMap::new(),
            body: None,
        }
    }

    /// Sets the method.
    pub fn set_method(mut self, method: HttpMethod) -> Self {
        self.method = method;
        self
    }

    /// Adds a header pair.
    pub fn add_header(mut self, header_field: &str, value: &str) -> Self {
        self.headers
            .insert(header_field.to_owned(), value.to_owned());
        self
    }

    /// Sets the request body.
    pub fn set_body<T>(mut self, body: Option<T>) -> Self
    where
        T: AsRef<[u8]>,
    {
        self.body = body.map(|b| b.as_ref().to_vec());
        self
    }

    /// Sends the request.
    ///
    /// This is an asynchronous method which blocks the caller before the
    /// response is received.
    ///
    /// ## Errors
    ///
    /// This method returns [`Result::Err(JsValue)`] when the underlying
    /// request reports an error.
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
                console::error1(&err);
                defer_err_clone.resolve(err);
            })
            .into_js_value(),
        );

        // Send the request with an optional body.
        if let Some(body) = self.body {
            let body_buf = Buffer::from_bytes(&body);
            req.write(body_buf);
        }
        req.end();

        #[cfg(debug_assertions)]
        console::log2(&"request sent: ".into(), &req);

        // Wait for the response.
        let resp: IncomingMessage =
            match select(defer_resp.into_future(), defer_err.into_future()).await {
                Either::Left((Ok(resp), _)) => Ok(resp),
                Either::Right((Ok(err), _)) => Err(err),
                _ => unreachable!("Impossible code path"),
            }?
            .into();

        #[cfg(debug_assertions)]
        console::log2(&"response received: ".into(), &resp);

        Ok(HttpResponse::new(resp))
    }
}

/// A received HTTP response.
///
/// Values of this type is created via [`HttpRequest`], you can read
/// the body data from the stream returned from `HttpRequest::body()`
/// method.
pub struct HttpResponse {
    fut: Pin<Box<dyn Future<Output = Result<(), JsValue>>>>,
    data_stream: AsyncIter<Buffer>,
    resp: IncomingMessage,
}

impl HttpResponse {
    fn new(resp: IncomingMessage) -> Self {
        let data_stream = AsyncIter::new();

        let mut data_stream_sender = data_stream.sender();
        resp.on(
            "data",
            closure!(|chunk: Buffer| {
                #[cfg(debug_assertions)]
                console::log_str("chunk received");
                data_stream_sender.send(Some(chunk));
            })
            .into_js_value(),
        );

        let mut data_stream_sender_for_end = data_stream.sender();
        let defer_end = Defer::new();
        let defer_end_clone = defer_end.clone();
        resp.on(
            "end",
            closure_once!(|| {
                #[cfg(debug_assertions)]
                console::log_str("response ended");
                data_stream_sender_for_end.send(None);
                defer_end_clone.resolve(JsValue::UNDEFINED);
            })
            .into_js_value(),
        );

        let mut data_stream_sender_for_error = data_stream.sender();
        let defer_err = Defer::new();
        let defer_err_clone = defer_err.clone();
        resp.on(
            "error",
            closure_once!(|err: JsValue| {
                console::error1(&err);
                data_stream_sender_for_error.send(None);
                defer_err_clone.resolve(err);
            })
            .into_js_value(),
        );

        let fut = async move {
            match select(defer_end.into_future(), defer_err.into_future()).await {
                Either::Right((Ok(err), _)) => Err(err),
                _ => Ok(()),
            }
        };

        Self {
            fut: Box::pin(fut),
            data_stream,
            resp,
        }
    }

    /// Returns the status code of this response.
    pub fn status_code(&self) -> u16 {
        self.resp.status_code()
    }

    /// Returns an [`AsyncIter<Buffer>`] for reading data of the body.
    pub fn body(&mut self) -> &mut AsyncIter<Buffer> {
        &mut self.data_stream
    }

    /// Returns the body as a string.
    pub async fn text(&mut self) -> String {
        self.body()
            .map(|chunk| chunk.to_string("utf-8"))
            .collect::<Vec<_>>()
            .await
            .join("")
    }
}

impl Drop for HttpResponse {
    fn drop(&mut self) {
        #[cfg(debug_assertions)]
        console::log2(&"response dropped: ".into(), &self.resp);

        self.resp.destroy(None);
    }
}

impl Future for HttpResponse {
    type Output = Result<(), JsValue>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let fut = self.fut.as_mut();
        fut.poll(cx)
    }
}
