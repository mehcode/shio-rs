extern crate shio;

use std::thread;
use std::sync::atomic::{AtomicUsize, Ordering};
use shio::prelude::*;

#[derive(Default)]
struct HandlerWithState {
    counter: AtomicUsize,
}

impl shio::Handler for HandlerWithState {
    type Result = String;

    fn call(&self, _: Context) -> Self::Result {
        let counter = self.counter.fetch_add(1, Ordering::Relaxed);

        format!(
            "Hi, #{} (from thread: {:?})\n",
            counter,
            thread::current().id()
        )
    }
}

fn main() {
    Shio::default()
        .route((Method::Get, "/", HandlerWithState::default()))
        .run(":7878")
        .unwrap();
}
