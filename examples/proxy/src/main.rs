//! Proxy google and stream the response back to the client.
//! This could be expanded into a simple http-proxy.

// TODO: Expand this a bit.
//  - Add some command line args: `proxy -p 7878 http://www.google.com`
//  - Add log and log when a request is proxied
//  - Use `Shio::new( .. )` as the router is just getting in the way here
//  - Proxy full request path

extern crate shio;
extern crate hyper;

use shio::prelude::*;
use hyper::Client;

fn proxy_google(ctx: Context) -> BoxFuture<Response, hyper::Error> {
    Client::new(&ctx)
        .get("http://www.google.com".parse().unwrap())
        .map(|res| Response::build().body(res.body()))
        .into_box()
}

fn main() {
    Shio::default()
        .route((Method::Get, "/", proxy_google))
        .run(":7878")
        .unwrap();
}
