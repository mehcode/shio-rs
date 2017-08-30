//! Proxy google and stream the response back to the client.
//! This could be expanded into a simple http-proxy.

#![feature(proc_macro, conservative_impl_trait, generators)]

// TODO: Expand this a bit.
//  - Add some command line args: `proxy -p 7878 http://www.google.com`
//  - Add log and log when a request is proxied
//  - Use `Shio::new( .. )` as the router is just getting in the way here
//  - Proxy full request path

extern crate hyper;
extern crate shio;
extern crate futures_await as futures;

use futures::prelude::*;
use shio::prelude::*;
use hyper::Client;

#[async]
fn proxy(ctx: Context) -> Result<Response, hyper::Error> {
    // Additional work can be scheduled on the thread-local event loop,
    // as each handler receives a reference to it
    let client = Client::new(ctx.handle());
    let res = await!(client.get("http://www.google.com".parse().unwrap()))?;

    // Map the _streaming_ response from google into a _streaming_
    // response from us
    Ok(Response::build().body(res.body()))
}

fn main() {
    // Our simple HTTP proxy doesn't need a Router and Shio doesn't force
    // a router on you.

    // `Shio::new` expects a single root handler. By default this
    // is a (wrapped) Router. As we don't need a router, we are using
    // `Shio::new` to specify our own root handler.

    Shio::new(proxy).run(":7878").unwrap();

    // Here is an example of what `Shio::default` is equivalent to:
    /*

    Shio::new(
      // Stack is a middleware container that executes each middleware
      // around _its_ root handler
      shio::Stack::new(shio::router::Router::new())
        // By default, the Recover middleware is included in your stack
        .with(shio::middleware::Recover)
    )

    */
}
