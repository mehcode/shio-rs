#![feature(conservative_impl_trait)]

extern crate salt;
extern crate futures;
extern crate tokio_io;
extern crate serde;

#[macro_use]
extern crate serde_json;

#[macro_use]
extern crate error_chain;

#[macro_use]
extern crate serde_derive;

mod errors {
    error_chain! {
        foreign_links {
            Io(::std::io::Error);
            Json(::serde_json::Error);
        }
    }
}

use tokio_io::io;
use futures::future;
use salt::prelude::*;
use errors::Error;

#[derive(Debug, Deserialize)]
struct RequestBody {
    visible: bool,
    name: String,
}

fn index(ctx: Context) -> impl Future<Item = Response, Error = Error> {
    io::read_to_end(ctx, Vec::new()).from_err().and_then(|(_, buffer)| {
        future::done(serde_json::from_slice(&buffer)).from_err()
    }).and_then(|body: RequestBody| {
        future::done(serde_json::to_string(&json!({
            "id": 20u8,
            "name": body.name,
        }))).from_err()
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
