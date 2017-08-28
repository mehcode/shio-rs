use std::sync::Arc;
use std::panic::AssertUnwindSafe;

use hyper;
use futures::{future, lazy, Future};

use ext::BoxFuture;
use handler::{default_catch, HandlerMut, BoxHandlerMut};
use response::Response;
use context::Context;
use middleware::Middleware;

/// Middleware that recovers from `panic!` anywhere in the stack, returning
/// an Internal Server Error (500) to the client.
///
/// This middleware is included in the default `Stack` as the first middleware.
/// When including this in a custom `Stack`, be sure to include this first as well or
/// `panic!`s in previous middleware will not be caught.
///
/// ```rust
/// # use shio::{Stack, middleware};
/// // Custom Stack with Recover middleware
/// Stack::new(|_| { /* [...] */ }).with(middleware::Recover);
///
/// // Default Stack with Recover middleware
/// Stack::default();
/// ```
pub struct Recover;

impl Middleware for Recover {
    fn call(&self, next: BoxHandlerMut) -> BoxHandlerMut {
        Box::new(RecoverHandler(next))
        // let next = Arc::new(next);
        // Box::new(move |ctx: Context| -> BoxFuture<Response, hyper::Error> {
        //     let next = next.clone();
        //     Box::new(
        //         AssertUnwindSafe(lazy(move || next.call(ctx)))
        //             .catch_unwind()
        //             .then(move |result| -> BoxFuture<Response, hyper::Error> {
        //                 Box::new(match result {
        //                     Err(err) => future::ok(default_catch(err)),
        //                     Ok(result) => future::result(result),
        //                 })
        //             }),
        //     )
        // })
    }
}

struct RecoverHandler(BoxHandlerMut);

impl HandlerMut for RecoverHandler {
    type Result = BoxFuture<Response, hyper::Error>;

    fn call(&self, context: &mut Context) -> Self::Result {
        self.0.call(context)
    }
}
