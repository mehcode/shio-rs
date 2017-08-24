use std::sync::Arc;
use std::fmt;

use hyper;
use tokio_core::reactor::Handle;
use futures::{Future, IntoFuture};

use response::Response;
use handler::{default_catch, Handler};
use context::Context;

// FIXME: Why does #[derive(Clone)] not work here? This _seems_ like a implementation that
//        should be auto-derived.

// #[derive(Clone)]
pub(crate) struct Service<H: Handler + 'static>
where
    <H::Result as IntoFuture>::Error: fmt::Debug + Send + Sync,
{
    handler: Arc<H>,
    handle: Handle,
}

impl<H: Handler + 'static> Service<H>
where
    <H::Result as IntoFuture>::Error: fmt::Debug + Send + Sync,
{
    pub(crate) fn new(handler: Arc<H>, handle: Handle) -> Self {
        Service { handler, handle }
    }
}

impl<H: Handler + 'static> Clone for Service<H>
where
    <H::Result as IntoFuture>::Error: fmt::Debug + Send + Sync,
{
    fn clone(&self) -> Self {
        Service {
            handler: self.handler.clone(),
            handle: self.handle.clone(),
        }
    }
}

impl<H: Handler + 'static> hyper::server::Service for Service<H>
where
    <H::Result as IntoFuture>::Error: fmt::Debug + Send + Sync,
{
    type Request = hyper::Request;
    type Response = hyper::Response;
    type Error = hyper::Error;
    type Future = Box<Future<Item = Self::Response, Error = Self::Error>>;

    fn call(&self, request: Self::Request) -> Self::Future {
        let ctx = Context::new(request, self.handle.clone());

        Box::new(
            self.handler
                .call(ctx)
                .into_future()
                .or_else(default_catch)
                .map(Response::into_hyper_response),
        )
    }
}
