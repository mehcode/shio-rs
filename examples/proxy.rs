#![feature(conservative_impl_trait)]

extern crate shio;
extern crate hyper;

use shio::prelude::*;
use hyper::Client;

// Handlers can return a `shio::Response` or an `impl Future<Item = shio::Response>` (
// which `impl FutureResponse` is an alias for).
fn proxy_google(
    ctx: Context,
) -> impl Future<Item = Response, Error = impl ::std::fmt::Debug + Send> {
    // Proxy google and stream the response back to the client
    // This could easily be expanded into a simple http-proxy
    Client::new(&ctx)
        .get("http://www.google.com".parse().unwrap())
        .map(|res| Response::build().body(res.body()))
}

fn main() {
    Shio::default()
        .route((Method::Get, "/", proxy_google))
        .run(":7878")
        .unwrap();
}
