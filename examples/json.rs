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

// TODO: Contribute this as serde_json::from_async_reader
// fn json_from_async_reader<'a, T: serde::de::Deserialize<'a>, R: io::AsyncRead>(reader: R) -> T {
//     io::read_to_end(reader, Vec::new()).from_err::<Error>().and_then(|(_, buffer)| {
//         future::done(serde_json::from_slice(&buffer)).from_err()
//     })
// }

#[derive(Debug, Deserialize)]
struct RequestBody {
    visible: bool,
    name: String,
}

fn index(ctx: Context) -> impl Future<Item = Response, Error = Error> {
    // TODO: Use serde_json::from_async_reader(..) when available
    // serde_json::from_async_reader(ctx).map(|(_, body: RequestBody)| {

    io::read_to_end(ctx, Vec::new()).from_err().and_then(|(_, buffer)| {
        future::done(serde_json::from_slice(&buffer)).from_err()
    }).map(|body: RequestBody| {
        // TODO: Add a Responder so we can do:
        //       Decide if we should have `Json` in a `salt_contrib` or not
        //   Response::with(Json(json!( ... )))
        //   Response::with(Json( ... ))

        Response::with(serde_json::to_string(&json!({
            "id": 20u8,
            "name": body.name,
        }))).with_header(header::ContentType::json())
    })
}

fn main() {
    Salt::default()
        .route((Method::Post, "/", index))
        .run(":7878")
        .unwrap();
}
