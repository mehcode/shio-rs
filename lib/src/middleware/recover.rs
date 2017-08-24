//! Defines the recover middleware, that permit catch `panic!`
//! and return an internal error

use std::sync::Arc;

use handler::BoxHandler;
use context::Context;
use response::{Response, BoxFutureResponse};

use std::panic::AssertUnwindSafe;
use futures::{lazy, future, Future};
use hyper::StatusCode;
use handler::default_catch;

/// Catch `panic!`, and send back an error 500.
pub(super) fn recover_panics(next: BoxHandler) -> BoxHandler {
    let next = Arc::new(next);
    Box::new(move |ctx: Context| -> BoxFutureResponse {
        let next = next.clone();
        Box::new(
            AssertUnwindSafe(lazy(move || {
                next.call(ctx)
            }))
            .catch_unwind()
            .then(move |result| -> BoxFutureResponse {
                Box::new(match result {
                    Err(err) => future::ok(default_catch(err)),
                    Ok(result) => future::result(result),
                })
            })
        )
    })
}
