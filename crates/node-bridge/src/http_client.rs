use std::collections::HashMap;
use std::fmt::{self, Debug};
use std::future::{Future, IntoFuture};
use std::pin::Pin;
use std::sync::{Arc, Weak};
use std::task::{Context, Poll};

use futures::future::{select, Either};
use js_sys::{Object as JsObject, Reflect};
use wasm_bindgen::prelude::*;

use crate::bindings::http2::{self, ClientHttp2Stream};
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
    body: Option<String>,
    body_bytes: Option<Vec<u8>>,
}

impl HttpRequest {
    /// Constructs a new request.
    pub fn new(url: &str) -> Self {
        Self {
            url: url.to_owned(),
            method: HttpMethod::Get,
            headers: HashMap::new(),
            body: None,
            body_bytes: None,
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

    /// Sets the body string.
    pub fn set_body(mut self, body: String) -> Self {
        self.body = Some(body);
        self
    }

    /// Sets the body bytes
    pub fn set_body_bytes(mut self, bytes: Vec<u8>) -> Self {
        self.body_bytes = Some(bytes);
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
            let body_buf = Buffer::from_str(&body, "utf-8");
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

    pub async fn send_as_http2(self) -> Result<Http2Response, JsValue> {
        let client = http2::connect(&self.url);

        let headers = JsObject::new();
        for (header_field_name, header_field_value) in &self.headers {
            Reflect::set(
                &headers,
                &header_field_name.into(),
                &header_field_value.into(),
            )?;
        }
        #[cfg(debug_assertions)]
        console::log2(&"request headers: {}".into(), &headers);

        let req = client.request(headers.into());

        if let Some(body_bytes) = self.body_bytes {
            req.write(Buffer::from_array(body_bytes));
            req.end();
        }

        Ok(Http2Response::new(req))
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

pub struct Http2Response {
    fut: Pin<Box<dyn Future<Output = Result<(), JsValue>>>>,
    data_stream: AsyncIter<Vec<u8>>,
    resp: ClientHttp2Stream,
}

impl Http2Response {
    fn new(resp: ClientHttp2Stream) -> Self {
        let resp = Arc::new(resp);

        let data_stream = AsyncIter::new();
        let mut data_stream_sender = data_stream.sender();

        resp.on(
            "data",
            closure!(|chunk: Vec<u8>| {
                #[cfg(debug_assertions)]
                console::log_str("chunk received");
                data_stream_sender.send(Some(chunk));
            })
            .into_js_value(),
        );

        let mut data_stream_sender_for_end = data_stream.sender();
        let defer_end = Defer::new();
        let defer_end_clone = defer_end.clone();
        let resp_clone = Arc::downgrade(&resp);

        resp.on(
            "end",
            closure_once!(|| {
                #[cfg(debug_assertions)]
                console::log_str("response ended");
                data_stream_sender_for_end.send(None);
                defer_end_clone.resolve(JsValue::UNDEFINED);
                Self::need_extra_close(resp_clone);
            })
            .into_js_value(),
        );

        let mut data_stream_sender_for_error = data_stream.sender();
        let defer_err = Defer::new();
        let defer_err_clone = defer_err.clone();
        let resp_clone = Arc::downgrade(&resp);

        resp.on(
            "error",
            closure_once!(|err: JsValue| {
                console::error1(&err);
                data_stream_sender_for_error.send(None);
                defer_err_clone.resolve(err);
                Self::need_extra_close(resp_clone);
            })
            .into_js_value(),
        );

        let fut = async move {
            match select(defer_end.into_future(), defer_err.into_future()).await {
                Either::Right((Ok(err), _)) => Err(err),
                _ => Ok(()),
            }
        };

        // whether this is a http2 error, resp_clone will be dropped,
        // as a result, resp will be the only pointer here, we can
        // access try_unwrap method. just in case, If unwrap is failed,
        // program will panic and print the Debug message.

        Self {
            fut: Box::pin(fut),
            data_stream,
            resp: Arc::try_unwrap(resp).unwrap(),
        }
    }

    /// Returns an [`AsyncIter<Buffer>`] for reading data of the body.
    pub fn body(&mut self) -> &mut AsyncIter<Vec<u8>> {
        &mut self.data_stream
    }

    fn need_extra_close(resp: Weak<ClientHttp2Stream>) -> bool {
        resp.upgrade()
            .and_then(|resp| {
                resp.session().close();
                Some(true)
            })
            .unwrap_or(false)
    }
}

impl Debug for ClientHttp2Stream {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&"error when unwrap ClientHttp2Stream in Http2Response::new method")
    }
}

impl Drop for Http2Response {
    fn drop(&mut self) {
        #[cfg(debug_assertions)]
        console::log2(&"response dropped: ".into(), &self.resp);

        self.resp.destroy();
    }
}

impl Future for Http2Response {
    type Output = Result<(), JsValue>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let fut = self.fut.as_mut();
        fut.poll(cx)
    }
}
