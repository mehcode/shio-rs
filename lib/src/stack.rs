use std::sync::Arc;
use std::fmt;
use std::marker::PhantomData;
use std::mem;

use hyper;

use response::Response;
use context::Context;
use handler::{BoxHandlerMut, HandlerMut};
use middleware::{self, BoxMiddleware, Middleware};
use router::Router;
use ext::{BoxFuture, IntoFutureExt};

pub struct Stack<H: HandlerMut + 'static>
where
    <H::Result as IntoFutureExt<Response>>::Error: fmt::Debug + Send + Sync,
{
    handler: Option<BoxHandlerMut>,
    handler_ptr: *const H,
    phantom: PhantomData<H>,
}

unsafe impl<H: HandlerMut + 'static> Send for Stack<H>
where
    <H::Result as IntoFutureExt<Response>>::Error: fmt::Debug + Send + Sync
{
}

unsafe impl<H: HandlerMut + 'static> Sync for Stack<H>
where
    <H::Result as IntoFutureExt<Response>>::Error: fmt::Debug + Send + Sync
{
}

impl<H: HandlerMut + 'static> Stack<H>
where
    <H::Result as IntoFutureExt<Response>>::Error: fmt::Debug + Send + Sync,
{
    pub fn new(handler: H) -> Self {
        let handler = Box::new(::handler::BoxedHandlerMut(handler));
        let handler_ptr: *const *const H = unsafe { mem::transmute(&handler) };

        Stack {
            handler: Some(handler),
            handler_ptr: unsafe { *handler_ptr },
            phantom: PhantomData,
        }
    }

    pub(crate) fn handler(&mut self) -> &mut H {
        unsafe { mem::transmute(self.handler_ptr) }
    }

    pub(crate) fn add<T: Middleware + 'static>(&mut self, middleware: T) {
        self.handler = Some(middleware.call(self.handler.take().unwrap()));
    }

    pub fn with<T: Middleware + 'static>(mut self, middleware: T) -> Self {
        self.add(middleware);
        self
    }
}

impl<H: HandlerMut + 'static> HandlerMut for Stack<H>
where
    <H::Result as IntoFutureExt<Response>>::Error: fmt::Debug + Send + Sync,
{
    type Result = BoxFuture<Response, hyper::Error>;

    #[inline]
    fn call(&self, ctx: &mut Context) -> Self::Result {
        self.handler.as_ref().unwrap().call(ctx)
    }
}

impl Default for Stack<Router> {
    fn default() -> Self {
        Stack::new(Router::new()).with(middleware::Recover)
    }
}
