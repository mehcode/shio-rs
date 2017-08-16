#![feature(conservative_impl_trait)]

// [x] Fix response type problem for handlers
// [ ] Make generic over "Handler" (and have default be with the router)
// [x] Expose request properties on context
// [ ] Provide `Context::spawn` (threadpool)
// [ ] Bind all addresses
// [x] Multi-threaded
// [x] Remove usage of `http` crate (until hyper switches)

extern crate salt;
extern crate hyper;
extern crate futures;

use std::thread;
use std::sync::Mutex;
use salt::prelude::*;
use hyper::Client;

fn index(_: Context) -> Response {
    // Simple requests should be simple
    Response::new().body("Hello World!\n")
}

// Handlers can return a `salt::Response` or an `impl Future<Item = salt::Response>` (
// which `impl FutureResponse` is an alias for). A `Future<Item = hyper::Response>` is
// coercible into a `salt::Response` (which is what is happening in this handler).
fn proxy_google(ctx: Context) -> impl FutureResponse {
    // Proxy google and stream the response back to the client
    // This could easily be expanded into a simple http-proxy
    Client::new(&ctx).get("http://www.google.com".parse().unwrap())
}

#[derive(Default)]
struct HandlerWithState {
    // NOTE: Locking a mutex on each request is quite bad for performance. Something like
    //       crossbeam might be a better fit here.
    // https://github.com/crossbeam-rs/crossbeam
    index: Mutex<usize>,
}

impl salt::Handler for HandlerWithState {
    type Future = Response;

    fn call(&self, _: Context) -> Self::Future {
        let mut index = self.index.lock().unwrap();
        *index += 1;

        Response::new().body(format!("Hi, #{} (from thread: {:?})\n", index, thread::current().id()))
    }
}

fn main() {
    let mut s = Salt::new();

    s.add((Method::Get, "/", index));
    s.add((Method::Get, "/google", proxy_google));
    s.add((Method::Get, "/hi", HandlerWithState::default()));

    // Set the number of threads to use.
    // By default, this is 1 in debug builds and <num_cpus> in release builds.
    s.threads(8);

    // Run the server indefinitely on the given address.
    s.run("127.0.0.1:7878");
}
