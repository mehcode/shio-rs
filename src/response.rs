use futures::{Async, Future, Poll};
use hyper;

use responder::Responder;

/// Represents an HTTP response.
pub struct Response {
    inner: Option<hyper::Response<hyper::Body>>,
}

impl Response {
    pub fn new() -> Response {
        Default::default()
    }

    pub fn with<R: Responder>(responder: R) -> Response {
        responder.to_response()
    }

    /// Set the `StatusCode` for this response.
    #[inline]
    pub fn status(mut self, status_code: hyper::StatusCode) -> Self {
        self.inner.as_mut().map(|response| {
            response.set_status(status_code);
        });

        self
    }

    /// Set the body for this `Response`.
    #[inline]
    pub fn body<B: Into<hyper::Body>>(mut self, body: B) -> Self {
        self.inner.as_mut().map(|response| {
            response.set_body(body.into());
        });

        self
    }

    /// Set a [`Header`] for this `Response`.
    #[inline]
    pub fn header<H: hyper::header::Header>(mut self, header: H) -> Self {
        self.inner.as_mut().map(|response| {
            response.headers_mut().set(header)
        });

        self
    }
}

impl Default for Response {
    fn default() -> Self {
        Response {
            inner: Some(hyper::Response::new()),
        }
    }
}

impl Future for Response {
    type Item = hyper::Response<hyper::Body>;
    type Error = hyper::Error;

    #[inline]
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        Ok(Async::Ready(self.inner.take().expect("cannot poll salt::Response twice")))
    }
}

pub type BoxFutureResponse = Box<Future<Item = hyper::Response, Error = hyper::Error>>;

/// Trait alias for `Future<Item = hyper::Response>`.
///
/// This looks weird because we can't use normal type aliases in a trait bound. Waiting on
/// https://github.com/rust-lang/rust/issues/41517.
pub trait FutureResponse: Future<Item = hyper::Response> {
}

impl<F: Future<Item = hyper::Response, Error = hyper::Error>> FutureResponse for F {
}
