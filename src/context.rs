use std::ops::Deref;

use hyper::Request;
use tokio_core::reactor::Handle;

pub struct Context {
    request: Request,
    handle: Handle,
}

impl Context {
    pub(crate) fn new(request: Request, handle: Handle) -> Self {
        Context { request, handle }
    }

    pub(crate) fn request(&self) -> &Request {
        &self.request
    }
}

impl Deref for Context {
    type Target = Handle;

    fn deref(&self) -> &Self::Target {
        &self.handle
    }
}
