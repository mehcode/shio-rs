use futures::{Future, IntoFuture};
use futures::future::{self, FutureResult};
use hyper;

use responder::Responder;

/// Represents an HTTP response.
pub struct Response {
    inner: hyper::Response<hyper::Body>,
}

impl Response {
    pub fn new() -> Response {
        Default::default()
    }

    pub fn with<R: Responder>(responder: R) -> Response {
        responder.to_response()
    }

    pub(crate) fn into_hyper_response(self) -> hyper::Response<hyper::Body> {
        self.inner
    }

    /// Get the status.
    #[inline]
    pub fn status(&self) -> hyper::StatusCode {
        self.inner.status()
    }

    /// Set the status and move the `Response`.
    #[inline]
    pub fn with_status(mut self, status_code: hyper::StatusCode) -> Self {
        self.inner.set_status(status_code);
        self
    }

    /// Take the body.
    pub fn body(self) -> hyper::Body {
        self.inner.body()
    }

    /// Set the body and move the `Response`.
    #[inline]
    pub fn with_body<B: Into<hyper::Body>>(mut self, body: B) -> Self {
        self.inner.set_body(body);
        self
    }

    /// Set a `Header` and move the `Response`.
    #[inline]
    pub fn with_header<H: hyper::header::Header>(mut self, header: H) -> Self {
        self.inner.headers_mut().set(header);
        self
    }
}

impl Default for Response {
    fn default() -> Self {
        Response {
            inner: hyper::Response::new(),
        }
    }
}

impl IntoFuture for Response {
    type Item = Response;
    type Error = hyper::Error;
    type Future = FutureResult<Self::Item, Self::Error>;

    #[inline]
    fn into_future(self) -> Self::Future {
        future::ok(self)
    }
}

pub type BoxFutureResponse = Box<Future<Item = Response, Error = hyper::Error>>;

/// Trait alias for `Future<Item = Response>`.
///
/// This looks weird because we can't use normal type aliases in a trait bound. Waiting on
/// https://github.com/rust-lang/rust/issues/41517.
pub trait FutureResponse: Future<Item = Response> {}

impl<F: Future<Item = Response, Error = hyper::Error>> FutureResponse for F {}
