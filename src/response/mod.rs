mod builder;

pub use self::builder::Builder;

use futures::{Future, IntoFuture};
use futures::future::{self, FutureResult};
use hyper;

use responder::Responder;
use StatusCode;
use header::Headers;

/// Represents an HTTP response.
pub struct Response {
    inner: hyper::Response<hyper::Body>,
}

impl Response {
    pub fn new() -> Response {
        Default::default()
    }

    /// Creates a new builder-style object to manufacture a Response.
    ///
    pub fn build() -> Builder {
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
    pub fn status(&self) -> StatusCode {
        self.inner.status()
    }

    /// Set the [`StatusCode`] for this response.
    ///
    /// [`StatusCode`]: ../enum.StatusCode.html
    #[inline]
    pub fn set_status(&mut self, status: StatusCode) {
        self.inner.set_status(status);
    }

    /// Take the body.
    pub fn body(self) -> hyper::Body {
        self.inner.body()
    }

    /// Set the body for this response.
    #[inline]
    pub fn set_body<B: Into<hyper::Body>>(&mut self, body: B) {
        self.inner.set_body(body.into());
    }

    /// Get a mutable reference to the headers.
    #[inline]
    pub fn headers_mut(&mut self) -> &mut Headers {
        self.inner.headers_mut()
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
