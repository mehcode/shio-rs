use futures::{Async, Future, Poll};
use hyper;

// TODO: Look into simplifying this. I don't like the .take in the Future impl
//       and the builder impls are very verbose. Should probably use the type system more and
//       have the compiler yell if someone tries to set the body twice (for instance)

/// Represents an HTTP response.
pub struct Response {
    inner: Option<hyper::Response<hyper::Body>>,
}

impl Response {
    pub fn new() -> Response {
        Response {
            inner: Some(hyper::Response::new()),
        }
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
}

impl Future for Response {
    type Item = hyper::Response<hyper::Body>;
    type Error = hyper::Error;

    #[inline]
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        Ok(Async::Ready(self.inner.take().expect("cannot poll salt::Response twice")))
    }
}

/// Trait alias for `Future<Item = hyper::Response>`.
///
/// This looks weird because we can't use normal type aliases in a trait bound. Waiting on
/// https://github.com/rust-lang/rust/issues/41517.
pub trait FutureResponse: Future<Item = hyper::Response> {
}

impl<F: Future<Item = hyper::Response, Error = hyper::Error>> FutureResponse for F {
}
