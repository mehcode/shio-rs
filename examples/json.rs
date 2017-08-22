#![feature(conservative_impl_trait)]

extern crate salt;
extern crate futures;
extern crate serde;

#[macro_use]
extern crate serde_json;

#[macro_use]
extern crate serde_derive;

use futures::future;
use salt::prelude::*;

#[derive(Debug, Deserialize)]
struct RequestBody {
    visible: bool,
    name: String,
}

fn index<'a>(ctx: Context) -> impl Future<Item = Response> + 'a {
    serde_json::from_async_reader(ctx).and_then(|(_, body): (_, RequestBody)| {
        future::done(serde_json::to_string(&json!({
            "id": 20u8,
            "name": body.name,
        })))
    }).map(|s| {
        Response::with(s).with_header(header::ContentType::json())
    })
}

fn main() {
    Salt::default()
        .route((Method::Post, "/", index))
        .run(":7878")
        .unwrap();
}
