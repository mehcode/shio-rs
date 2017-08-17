extern crate salt;

use salt::prelude::*;

fn index(_: Context) -> Response {
    // Simple requests should be simple
    Response::new().body("Hello World!\n")
}

fn main() {
    let mut s = Salt::default();

    s.add((Method::Get, "/", index));

    // Set the number of threads to use.
    // By default, this is 1 in debug builds and <num_cpus> in release builds.
    s.threads(8);

    // Run the server indefinitely on the given address.
    s.run("127.0.0.1:7878");
}
