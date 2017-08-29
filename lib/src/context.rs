use std::ops::Deref;

use tokio_core::reactor::Handle;
use typemap::{Key, TypeMap};

use request::{Body, Request};

/// `Context` represents the context of the current HTTP request.
///
/// A `Context` consists of:
///     - The current HTTP [`Request`].
///     - A [`Handle`] referencing the event loop in which this request is being
///       handled.
///
/// [`Handle`]: https://docs.rs/tokio-core/0.1/tokio_core/reactor/struct.Handle.html
/// [`Request`]: ../request/struct.Request.html
pub struct Context {
    state: TypeMap,
    handle: Handle,
    request: Request,
}

impl Context {
    pub(crate) fn new(handle: Handle, request: Request) -> Self {
        Context {
            handle,
            request,
            state: TypeMap::new(),
        }
    }

    /// Return a reference to a handle to the event loop this `Context` is associated with.
    #[inline]
    pub fn handle(&self) -> &Handle {
        &self.handle
    }

    /// Take the request body.
    pub fn body(self) -> Body {
        self.request.body()
    }

    /// Puts a value into the request context.
    pub fn put<K: Key>(&mut self, value: K::Value) {
        self.state.insert::<K>(value);
    }

    /// Gets a value from the request context.
    pub fn get<K: Key>(&self) -> Option<&K::Value> {
        self.state.get::<K>()
    }
}

impl Deref for Context {
    type Target = Request;

    fn deref(&self) -> &Self::Target {
        &self.request
    }
}
