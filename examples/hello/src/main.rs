extern crate shio;

use shio::prelude::*;

// Simple requests should be simple, even in the face of asynchronous design.
fn index(_: Context) -> Response {
    // `Response::with( ... )` accepts an instance that implements `shio::Responder`
    // The implementation for &str will set the body of the response and
    // the Content-Length header.
    Response::with("Hello World!\n")

    // This would be equivalent:

    /*
    const PHRASE: &str = "Hello World\n";

    Response::build()
        .header(header::ContentLength(PHRASE.len() as u64))
        .body(PHRASE)
    */

    // The default status code is `Status::Ok` (200).
}

fn main() {
    // Construct a _default_ `Shio` service, mount the `index` handler, and
    // run indefinitely on port `7878` (by default, binds to both `0.0.0.0` and `::0`).
    Shio::default()
        .route((Method::Get, "/", index))
        .run(":7878")
        .unwrap();

    // Shio services have an entry `Handler` that must be defined.
    // `Shio::default` constructs a `Shio` service with `shio::router::Router`
    // as its entry `Handler`.

    // This would be equivalent:

    /*

    let mut router = shio::router::Router::new();

    router.route((Method::Get, "/", index));

    let mut service = Shio::new(router);

    service.run(":7878").unwrap();

    */
}
