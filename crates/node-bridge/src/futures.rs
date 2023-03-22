use std::cell::RefCell;
use std::collections::VecDeque;
use std::future::IntoFuture;
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context, Poll};

use futures::{Future, Stream};
use js_sys::{Function, Promise};
use pin_project_lite::pin_project;
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

        Self { resolver, promise }
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

pub struct AsyncIter<T> {
    inner: AsyncIterHandle<T>,
}

type AsyncIterHandle<T> = Rc<RefCell<AsyncIterInner<T>>>;

pub struct AsyncIterSender<T> {
    inner: AsyncIterHandle<T>,
}

pin_project! {
    pub struct AsyncIterInner<T> {
        ready_values: VecDeque<Option<T>>,
        defer_next: Option<Defer>,
        #[pin]
        active_fut: Option<JsFuture>,
    }
}

impl<T> AsyncIter<T> {
    pub fn new() -> Self {
        Self {
            inner: Rc::new(RefCell::new(AsyncIterInner {
                ready_values: VecDeque::new(),
                defer_next: None,
                active_fut: None,
            })),
        }
    }

    pub fn sender(&self) -> AsyncIterSender<T> {
        AsyncIterSender {
            inner: Rc::clone(&self.inner),
        }
    }
}

impl<T> Stream for AsyncIter<T> {
    type Item = T;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut inner_mut = self.inner.borrow_mut();
        let inner_pinned = Pin::new(&mut *inner_mut);
        inner_pinned.poll_next(cx)
    }
}

impl<T> AsyncIterSender<T> {
    pub fn send(&mut self, value: Option<T>) {
        let mut inner_mut = self.inner.borrow_mut();
        inner_mut.ready_values.push_back(value);

        // Wake up one waiter.
        if let Some(defer) = inner_mut.defer_next.as_ref() {
            defer.resolve(JsValue::NULL);
            inner_mut.defer_next = None;
        }
    }
}

impl<T> Stream for AsyncIterInner<T> {
    type Item = T;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();

        if let Some(fut) = this.active_fut.as_mut().as_pin_mut() {
            if matches!(fut.poll(cx), Poll::Pending) {
                return Poll::Pending;
            }
            // Wake up from waiting, clear the defer.
            this.active_fut.set(None);
            *this.defer_next = None;
        }

        if let Some(value) = this.ready_values.pop_front() {
            return Poll::Ready(value);
        }

        let defer = Defer::new();
        *this.defer_next = Some(defer.clone());

        let fut = JsFuture::from(defer.promise);
        this.active_fut.set(Some(fut));
        let fut = this.active_fut.as_pin_mut().unwrap();

        // Register the callback.
        let poll = fut.poll(cx);
        assert!(matches!(poll, Poll::Pending));

        Poll::Pending
    }
}
