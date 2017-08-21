#![feature(conservative_impl_trait)]

extern crate salt;
extern crate hyper;

use salt::prelude::*;
use hyper::Client;

// Handlers can return a `salt::Response` or an `impl Future<Item = salt::Response>` (
// which `impl FutureResponse` is an alias for).
fn proxy_google(ctx: Context) -> impl Future<Item = Response> {
    // Proxy google and stream the response back to the client
    // This could easily be expanded into a simple http-proxy
    Client::new(&ctx)
        .get("http://www.google.com".parse().unwrap())
        .map(|res| Response::new().with_body(res.body()))
}

fn main() {
    Salt::default()
        .route((Method::Get, "/", proxy_google))
        .run(":7878")
        .unwrap();
}
