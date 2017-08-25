use std::sync::Arc;
use std::fmt;

use hyper;

use response::{Response,    BoxFutureResponse};
use context::Context;
use handler::Handler;
use middleware::{self, BoxMiddleware, Middleware};
use router::Router;
use ext::IntoFutureExt;

pub struct Stack<H: Handler + 'static>
where
    <H::Result as IntoFutureExt<Response>>::Error: fmt::Debug + Send + Sync,
{
    pub(crate) handler: Arc<H>,
    middlewares: Vec<BoxMiddleware>,
}

impl<H: Handler + 'static> Stack<H>
where
    <H::Result as IntoFutureExt<Response>>::Error: fmt::Debug + Send + Sync,
{
    pub fn new(handler: H) -> Self {
        Stack {
            handler: Arc::new(handler),
            middlewares: Vec::new(),
        }
    }

    pub(crate) fn add<T: Middleware + 'static>(&mut self, middleware: T) {
        self.middlewares.push(middleware.into_box());
    }

    pub fn with<T: Middleware + 'static>(mut self, middleware: T) -> Self {
        self.add(middleware);
        self
    }
}

impl<H: Handler + 'static> Handler for Stack<H>
where
    <H::Result as IntoFutureExt<Response>>::Error: fmt::Debug + Send + Sync,
{
    type Result = BoxFutureResponse<hyper::Error>;

    #[inline]
    fn call(&self, ctx: Context) -> Self::Result {
        // Define the initial 'next' fn that simply calls the handler
        let handler = self.handler.clone();
        let mut next = (move |ctx: Context| handler.call(ctx)).into_box();

        // Iterate backwards through our stack to produce a forwards chain of 'next' fns
        for middleware in self.middlewares.iter().rev() {
            next = middleware.call(next);
        }

        // Kick off
        next.call(ctx)
    }
}

impl Default for Stack<Router> {
    fn default() -> Self {
        Stack::new(Router::new()).with(middleware::Recover)
    }
}
