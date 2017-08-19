extern crate salt;

use salt::prelude::*;

// Simple requests should be simple, even in the face of asynchronous design.
fn index(_: Context) -> Response {
    // `Response::with( ... )` accepts an instance that implements `salt::Responder`
    // The implementation for &str will set the body of the response and
    // the Content-Length header.
    Response::with("Hello World!\n")

    // This would be equivalent:

    /*
    const PHRASE: &str = "Hello World\n";

    Response::new()
        .body(PHRASE)
        .header(header::ContentLength(PHRASE.len() as u64))
    */

    // The default status code is `Status::Ok` (200).
}

fn main() {
    // Construct a _default_ `Salt` service, mount the `index` handler, and
    // run indefinitely on port `7878` (by default, binds to both `0.0.0.0` and `::0`).
    Salt::default().route((Method::Get, "/", index)).run(":7878").unwrap();

    // Salt services have an entry `Handler` that must be defined.
    // `Salt::default` constructs a `Salt` service with `salt::Router` as its entry `Handler`.

    // This would be equivalent:

    /*

    let mut router = Salt::Router::new();

    router.mount((Method::Get, "/", index));

    let mut service = Salt::new(router);

    service.run(":7878").unwrap();

    */
}
