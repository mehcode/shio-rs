// NOTE: error-chain produces this warning on the nightly compiler
#![allow(unused_doc_comment)]

extern crate shio;
extern crate tokio_core;

#[macro_use]
extern crate error_chain;

use std::time::Duration;
use shio::prelude::*;
use tokio_core::reactor::Timeout;
use errors::{Error, Result};

mod errors {
    // We're using error-chain! here to showcase usage of it, even though we technically don't
    // _need_ to use it.
    //
    // NOTE: error-chain currently (as of 0.10) does not allow you to configure bounds
    //       Handlers require the error type to be `Send + Sync` and error-chain 0.8+ produces
    //       an error type that is `Send` only.
    error_chain! {
        foreign_links {
            Io(::std::io::Error);
        }
    }
}

const PHRASE: &'static str = "Hello World\n";

// Return a response
fn index1(_: Context) -> Response {
    // Response::with( .. ) takes a `Responder` and
    // returns a `Response` _or_ a `BoxFuture<Response, _>` depending on the specific
    // responder.
    Response::with(PHRASE)

    // The `Response` may also be built using the `ResponseBuilder`.
    // If you don't specify a "Content-Length" the response will be sent with a
    // chunked encoding.
    //
    // Response::build().header(header::ContentLength(PHRASE.len())).body(PHRASE)
}

// A `{Responder}` may be returned directly
fn index2(_: Context) -> &'static str {
    PHRASE
}

// `Result<{Responder}, _>` implements Responder
fn index3(_: Context) -> Result<&'static str> {
    Ok(PHRASE)
}

// As does `Result<Response, _>`
fn index4(_: Context) -> Result<Response> {
    Ok(Response::with(PHRASE))
}

// When returning a _future_ as you will most likely be doing,
// the type is named as `BoxFuture< Response, _ >`
//
// A `.into_box` trait method is implemented for all futures to assist with
// wrapping the future in a box
fn index5(ctx: Context) -> BoxFuture<Response, Error> {
    Timeout::new(Duration::from_secs(2), &ctx)
        // `Timeout::new` returns a `Result<{Future}, io::Error>`
        // `.into_future` turns a `Result<T, E>` into a `{Future<Item = T, Error = E>}`
        .into_future()
        // `.from_err` then coerces the error into our final error type which is
        // from error-chain and implements `From<io::Error>`
        .from_err()
        .map(|_| Response::with(PHRASE))
        // `.and_then` consumes the result from the previous future and produces a future
        // `.map` consumes the result and produces a non-future value
        //
        // .and_then(|_| futures::future::ok(Response::with(PHRASE)))
        .into_box()
}

// When returning a _future_ you may also return a {Responder} directly
fn index6(ctx: Context) -> BoxFuture<&'static str, Error> {
    Timeout::new(Duration::from_secs(2), &ctx)
        .into_future()
        .from_err()
        .map(|_| PHRASE)
        .into_box()
}

// NOTE: The goal is to make `.into_box` and `BoxFuture` optional when `impl Trait` reaches
//       stable Rust.

fn main() {
    Shio::default()
        .route((Method::Get, "/1", index1))
        .route((Method::Get, "/2", index2))
        .route((Method::Get, "/3", index3))
        .route((Method::Get, "/4", index4))
        .route((Method::Get, "/5", index5))
        .route((Method::Get, "/6", index6))
        .run(":7878")
        .unwrap();
}
