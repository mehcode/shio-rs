use futures::Future;
use hyper;
use std::sync::Arc;

use super::{Handler, BoxHandler, BoxFutureResponse, Context};

// TODO: Discuss alternative designs to reduce allocation. We're knee deep in Boxes here.
//       I could just be missing something obvious that would keep the design and reduce Boxes.

/// A set of [`StackHandler`]s that are executed in sequence **around** a _root_ `Handler`.
///
/// A `Stack` is constructed by wrapping a _root_ `Handler`.
///
/// ```rust, ignore
/// # use salt::{Stack, Response, Status};
/// let stack = Stack::new(|_| {
///   // [...]
/// # Response::new().status(Status::NoContent)
/// });
/// ```
pub struct Stack {
    root: Arc<BoxHandler>,
    elements: Vec<BoxStackBoxHandler>,
}

impl Stack {
    pub fn new<H: Handler + 'static>(root: H) -> Self {
        Stack { root: Arc::new(root.boxed()),
            elements: Vec::new()
        }
    }

    // Appends a new [`StackHandler`] to the end of this `Stack`.
    pub fn add<T: StackHandler + 'static>(&mut self, handler: T) {
        self.elements.push(boxed(handler));
    }
}

impl Handler for Stack {
    type Future = BoxFutureResponse;

    #[inline]
    fn call(&self, ctx: Context) -> Self::Future {
        // Define the initial 'next' fn that simply calls the root handler
        let root = self.root.clone();
        let mut next = Box::new(move |ctx| root.call(ctx)) as BoxHandler;

        // Iterate backwards through our stack to produce a forwards chain of 'next' fns
        for element in self.elements.iter().rev() {
            next = element.call(next);
        }

        // Kick off
        next.call(ctx)
    }
}

pub trait StackHandler: Send + Sync {
    type Handler: Handler + 'static;

    fn call(&self, next: BoxHandler) -> Self::Handler;
}

impl<TError, TFuture, THandler, TFn> StackHandler for TFn
where
    TFuture: Future<Item = hyper::Response, Error = TError>,
    THandler: Handler<Future = TFuture>,
    THandler: 'static,
    TFn: Send + Sync,
    TFn: Fn(BoxHandler) -> THandler,
{
    type Handler = THandler;

    #[inline]
    fn call(&self, next: BoxHandler) -> Self::Handler {
        (*self)(next)
    }
}

trait StackBoxHandler: Send + Sync {
    fn call(&self, next: BoxHandler) -> BoxHandler;
}

impl<TFn> StackBoxHandler for TFn
where
    TFn: Send + Sync,
    TFn: Fn(BoxHandler) -> BoxHandler,
{
    #[inline]
    fn call(&self, next: BoxHandler) -> BoxHandler {
        (*self)(next)
    }
}

type BoxStackBoxHandler = Box<StackBoxHandler>;

fn boxed<H: StackHandler + 'static>(handler: H) -> BoxStackBoxHandler {
    Box::new(move |next: BoxHandler| {
        handler.call(next).boxed()
    })
}
