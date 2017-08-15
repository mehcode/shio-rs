#![feature(conservative_impl_trait)]

// [x] Fix response type problem for handlers
// [ ] Make generic over "Handler" (and have default be with the router)
// [ ] Expose request properties on context
// [ ] Provide `Context::spawn` (threadpool)
// [ ] Bind all addresses
// [x] Multi-threaded

extern crate salt;
extern crate hyper;
extern crate futures;

use std::thread;
use std::sync::Mutex;
use salt::prelude::*;
use hyper::Client;

fn index(_: Context) -> Response {
    Response::new().body("Hello World!\n")
}

fn proxy_google(ctx: Context) -> impl FutureResponse {
    let google_url = "http://www.google.com".parse().unwrap();
    let google_request = Client::new(&ctx).get(google_url);

    google_request.and_then(|google_response| {
        Response::new().body(google_response.body())
    })
}

#[derive(Default)]
struct HandlerWithState {
    index: Mutex<usize>,
}

impl salt::Handler for HandlerWithState {
    type Future = Response;

    fn call(&self, ctx: Context) -> Self::Future {
        let mut index = self.index.lock().unwrap();
        *index += 1;

        Response::new().body(format!("Hi, #{} (from thread: {:?})\n", index, thread::current().id()))
    }
}

fn main() {
    let mut s = Salt::new();

    s.add((http::method::GET, "/", index));
    s.add((http::method::GET, "/google", proxy_google));
    s.add((http::method::GET, "/hi", HandlerWithState::default()));

    s.run("127.0.0.1:7878");
}
