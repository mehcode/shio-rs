use std::sync::Arc;
use std::fmt;
use std::marker::PhantomData;

use hyper;
use tokio_core::reactor::Handle;
use futures::{lazy, Future};

use response::Response;
use handler::{default_catch, HandlerMut};
use context::Context;
use ext::{FutureExt, IntoFutureExt};

// FIXME: Why does #[derive(Clone)] not work here? This _seems_ like a implementation that
//        should be auto-derived.

// #[derive(Clone)]
pub(crate) struct Service<H>
where
    H: HandlerMut<'static> + 'static,
    <H::Result as IntoFutureExt<'static, Response>>::Error: fmt::Debug + Send + Sync,
{
    handler: Arc<H>,
    handle: Handle,
}

impl<H> Service<H>
where
    H: HandlerMut<'static> + 'static,
    <H::Result as IntoFutureExt<'static, Response>>::Error: fmt::Debug + Send + Sync,
{
    pub(crate) fn new(handler: Arc<H>, handle: Handle) -> Self {
        Service { handler, handle }
    }
}

impl<H> Clone for Service<H>
where
    H: HandlerMut<'static> + 'static,
    <H::Result as IntoFutureExt<'static, Response>>::Error: fmt::Debug + Send + Sync,
{
    fn clone(&self) -> Self {
        Service {
            handler: self.handler.clone(),
            handle: self.handle.clone(),
            context: None,
        }
    }
}

impl<H> hyper::server::Service for Service<H>
where
    H: HandlerMut<'static> + 'static,
    <H::Result as IntoFutureExt<'static, Response>>::Error: fmt::Debug + Send + Sync,
{
    type Request = hyper::Request;
    type Response = hyper::Response;
    type Error = hyper::Error;
    type Future = Box<Future<Item = Self::Response, Error = Self::Error>>;

    fn call(&self, request: Self::Request) -> Self::Future {
        let handler = self.handler.clone();
        let handle = self.handle.clone();

        let mut ctx = Context::new(request, handle);

        // [!!!]
        // NOTE: We need to wrap the context handle so it lasts the whole future

        handler
            .call(&mut ctx)
            .into_future_ext()
            .or_else(default_catch)
            .map(Response::into_hyper_response)
            .into_box()
    }
}
