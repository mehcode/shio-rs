use std::ops::Deref;

use hyper::{Request, Method, Uri};
use tokio_core::reactor::Handle;

/// `Context` represents the context of the current HTTP request.
///
/// A `Context` consists of:
///     - A [`Handle`] referencing the event loop in which this request is being
///       handled.
///     - The current HTTP [`Request`].
///
/// [`Handle`]: https://docs.rs/tokio-core/0.1/tokio_core/reactor/struct.Handle.html
/// [`Request`]: http://doc.rust-lang.org/hyper/0.11/hyper/client/struct.Request.html
pub struct Context {
    request: Request,
    handle: Handle,
}

impl Context {
    pub(crate) fn new(request: Request, handle: Handle) -> Self {
        Context { request, handle }
    }

    /// Return a reference to a handle to the event loop this `Context` is associated with.
    #[inline]
    pub fn handle(&self) -> &Handle {
        &self.handle
    }

    /// Return a reference to the [`Request`] this `Context` was constructed from.
    ///
    /// [`Request`]: http://doc.rust-lang.org/hyper/0.11/hyper/client/struct.Request.html
    #[inline]
    pub fn request(&self) -> &Request {
        &self.request
    }

    /// Returns a reference to the associated HTTP method.
    #[inline]
    pub fn method(&self) -> &Method {
        self.request.method()
    }

    /// Returns a reference to the associated request URI.
    #[inline]
    pub fn uri(&self) -> &Uri {
        self.request.uri()
    }

    /// Returns a reference to the associated request path.
    #[inline]
    pub fn path(&self) -> &str {
        self.request.path()
    }
}

impl Deref for Context {
    type Target = Handle;

    fn deref(&self) -> &Self::Target {
        &self.handle
    }
}
