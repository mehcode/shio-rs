//! Simple 'Hello World'

extern crate salt;
extern crate tokio_io;

use tokio_io::io;

use salt::Salt;

fn main() {
    Salt::new(|c| io::write_all(c, "Hello World!\n"))
        .listen("localhost:7878")
        .unwrap();
}
