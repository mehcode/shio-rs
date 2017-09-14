use std::ops::Deref;
use std::sync::Arc;

use tokio_core::reactor::Handle;
use unsafe_any::UnsafeAny;

use request::{Body, Request};
use util::typemap::TypeMap;
pub use util::typemap::Key;

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
    global_state: Arc<TypeMap<UnsafeAny + Send + Sync>>,
}

impl Context {
    pub(crate) fn new(
        handle: Handle,
        request: Request,
        global_state: Arc<TypeMap<UnsafeAny + Send + Sync>>,
    ) -> Self {
        Self {
            handle,
            request,
            state: TypeMap::new(),
            global_state,
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
    ///
    /// # Panics
    ///
    /// If there is no value in the request context of the given type.
    pub fn get<K: Key>(&self) -> &K::Value {
        self.state.get::<K>().unwrap()
    }

    /// Gets a value from the request context.
    pub fn try_get<K: Key>(&self) -> Option<&K::Value> {
        self.state.get::<K>()
    }

    /// Gets a value from the global context.
    ///
    /// # Panics
    ///
    /// If there is no value in the global context of the given type.
    pub fn get_global<K: Key>(&self) -> &K::Value
    where
        <K as Key>::Value: Send + Sync,
    {
        self.global_state.get::<K>().unwrap()
    }

    /// Gets a value from the global context.
    pub fn try_get_global<K: Key>(&self) -> Option<&K::Value>
    where
        <K as Key>::Value: Send + Sync,
    {
        self.global_state.get::<K>()
    }
}

impl Deref for Context {
    type Target = Request;

    fn deref(&self) -> &Self::Target {
        &self.request
    }
}
