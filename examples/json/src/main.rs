#![feature(proc_macro, conservative_impl_trait, generators)]

#[allow(unused_extern_crates)]
extern crate serde;
extern crate serde_json;
extern crate shio;
extern crate tokio_io;
extern crate futures_await as futures;

#[macro_use]
extern crate error_chain;

#[macro_use]
extern crate serde_derive;

use tokio_io::io;
use futures::prelude::*;
use shio::prelude::*;
use errors::*;

mod errors {
    error_chain! {
        foreign_links {
            Io(::std::io::Error);
            Json(::serde_json::Error);
        }
    }
}

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

#[async]
fn index(ctx: Context) -> Result<Response> {
    // `tokio_io::io::read_to_end` will asynchronously read the request body, to completion,
    // and place it in the new vector.
    let (_, buffer) = await!(io::read_to_end(ctx.body(), Vec::new()))?;

    // Parse the body into JSON
    let body: RequestBody = serde_json::from_slice(&buffer)?;

    // Return a JSON structure (using some of the received JSON)
    let s = serde_json::to_string(&ResponseBody { id: 20, name: body.name })?;

    Ok(Response::build().header(header::ContentType::json()).body(s))
}

fn main() {
    Shio::default()
        .route((Method::Post, "/", index))
        .run(":7878")
        .unwrap();
}
