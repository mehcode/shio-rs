use std::sync::Arc;
use std::fmt;

use futures::IntoFuture;

use response::BoxFutureResponse;
use context::Context;
use handler::{BoxHandler, Handler};
use middleware::{BoxMiddleware, Middleware};

pub struct Stack {
    root: Arc<BoxHandler>,
    middlewares: Vec<BoxMiddleware>,
}

impl Stack {
    pub fn new<H: Handler + 'static>(root: H) -> Self
    where
        <H::Result as IntoFuture>::Error: fmt::Debug + Send,
    {
        Stack {
            root: Arc::new(root.boxed()),
            middlewares: Vec::new(),
        }
    }

    pub fn with<T: Middleware + 'static>(mut self, middleware: T) -> Self {
        self.middlewares.push(middleware.boxed());
        self
    }
}

impl Handler for Stack {
    type Result = BoxFutureResponse;

    #[inline]
    fn call(&self, ctx: Context) -> Self::Result {
        // Define the initial 'next' fn that simply calls the root handler
        let root = self.root.clone();
        let mut next = Box::new(move |ctx| root.call(ctx)) as BoxHandler;

        // Iterate backwards through our stack to produce a forwards chain of 'next' fns
        for middleware in self.middlewares.iter().rev() {
            next = middleware.call(next);
        }

        // Kick off
        next.call(ctx)
    }
}
