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

fn json_in_json_out(ctx: Context) -> BoxFutureResponse<Box<Error + Send + Sync>> {
    io::read_to_end(ctx, Vec::new())
        .from_err()
        .and_then(|(_, buffer)| {
            let body: RequestBody = serde_json::from_slice(&buffer)?;
            let s = serde_json::to_string(&json!({
                "id": 20u8,
                "name": body.name,
            }))?;

            let mut response = Response::with(s);
            response.headers_mut().set(header::ContentType::json());

            Ok(response)
        })
        .into_box()
}

fn main() {
    Shio::default()
        .route((Method::Post, "/", json_in_json_out))
        .run(":7878")
        .unwrap();
}
