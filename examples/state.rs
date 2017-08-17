extern crate salt;

use std::thread;
use std::sync::atomic::{AtomicUsize, Ordering};
use salt::prelude::*;

#[derive(Default)]
struct HandlerWithState {
    counter: AtomicUsize,
}

impl salt::Handler for HandlerWithState {
    type Future = Response;

    fn call(&self, _: Context) -> Self::Future {
        let counter = self.counter.fetch_add(1, Ordering::Relaxed);

        Response::new().body(format!("Hi, #{} (from thread: {:?})\n", counter, thread::current().id()))
    }
}

fn main() {
    let mut s = Salt::default();

    s.add((Method::Get, "/", HandlerWithState::default()));

    s.run("127.0.0.1:7878");
}
