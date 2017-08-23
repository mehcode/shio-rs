extern crate shio;
extern crate tokio_io;
extern crate serde;

#[macro_use]
extern crate serde_json;

#[macro_use]
extern crate serde_derive;

use std::error::Error;
use tokio_io::io;
use shio::prelude::*;

#[derive(Debug, Deserialize)]
struct RequestBody {
    visible: bool,
    name: String,
}

#[derive(Debug, Serialize)]
struct ResponseBody {
    id: u8,
    name: String,
}

// First, let's look at composing `tokio_io` with `serde_json` manually.

fn manual_static(ctx: Context) -> BoxFutureResponse<Box<Error + Send + Sync>> {
    // `tokio_io::io::read_to_end` will asynchronously read the request body, to completion,
    // and place it in the new vector.
    io::read_to_end(ctx, Vec::new())
        // `Future::from_err` acts like `?` in that it coerces the error type from
        // the future into the final error type
        .from_err()
        // `Future::and_then` can be used to merge an asynchronous workflow with a
        // synchronous workflow
        //
        // `read_to_end` resolves to a tuple of our reader ( `Context` ) and the buffer.
        .and_then(|(_, buffer)| /* -> Result<Response, Box<Error + Send + Sync>> */ {
            let body: RequestBody = serde_json::from_slice(&buffer)?;
            let s = serde_json::to_string(ResponseBody { id: 20, name: body.name })?;

            Ok(Response::build().header(header::ContentType::json()).body(s))
        })
        // Put our future inside a Box so we can name our return type
        // This part will go away once `impl Trait` is stablized in Rust
        .into_box()
}

fn main() {
    Shio::default()
        .route((Method::Post, "/", manual_static))
        .run(":7878")
        .unwrap();
}
