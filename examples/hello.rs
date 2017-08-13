#![feature(conservative_impl_trait)]

extern crate salt;
extern crate tokio_io;
extern crate tokio_core;
extern crate futures;

use salt::*;
use futures::future::Future;
use std::error::Error;
use tokio_io::io;
use tokio_core::net::TcpStream;

// fn accept<'a>(socket: TcpStream) -> impl Future<Error = impl Error> + 'a {
//     io::write_all(socket, b"Hello World!\n")
// }

fn main() {
    // Salt::new(accept).listen("localhost:7878").unwrap();
    Salt::new(|socket| io::write_all(socket, b"Hello World!\n"))
        .listen("localhost:7878")
        .unwrap();
}
