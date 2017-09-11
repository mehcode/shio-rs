extern crate shio;

use std::thread;
use std::sync::atomic::{AtomicUsize, Ordering};
use shio::prelude::*;
use shio::context::Key;

pub struct GlobalCounter;

impl Key for GlobalCounter {
    type Value = AtomicUsize;
}

fn hello(context: Context) -> Response {
    let counter = context
        .get_global::<GlobalCounter>()
        .fetch_add(1, Ordering::Relaxed);
    Response::with(format!(
        "Hi, #{} (from thread: {:?})\n",
        counter,
        thread::current().id()
    ))
}

fn main() {
    Shio::default()
        .manage::<GlobalCounter>(AtomicUsize::default())
        .route((Method::Get, "/", hello))
        .run(":7878")
        .unwrap();
}
