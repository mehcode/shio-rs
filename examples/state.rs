extern crate salt;

use std::thread;
use std::sync::atomic::{AtomicUsize, Ordering};
use salt::prelude::*;

#[derive(Default)]
struct HandlerWithState {
    counter: AtomicUsize,
}

impl salt::Handler for HandlerWithState {
    type Result = Response;

    fn call(&self, _: Context) -> Self::Result {
        let counter = self.counter.fetch_add(1, Ordering::Relaxed);

        Response::with(format!(
            "Hi, #{} (from thread: {:?})\n",
            counter,
            thread::current().id()
        ))
    }
}

fn main() {
    Salt::default()
        .route((Method::Get, "/", HandlerWithState::default()))
        .run(":7878")
        .unwrap();
}
