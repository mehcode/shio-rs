extern crate salt;
extern crate tokio_io;

use tokio_io::io;

use salt::Salt;

fn main() {
    Salt::new(|stream| io::write_all(stream, "Hello World!\n"))
        .listen("localhost:7878")
        .unwrap();
}
