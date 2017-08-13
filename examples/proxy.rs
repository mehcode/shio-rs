//! Dumb proxy that just connects to google's home page
//! With some command line arguments this could be expanded to a proper proxy

extern crate salt;
extern crate futures;
extern crate tokio_io;
extern crate hyper;

use tokio_io::io;
use hyper::Client;
use futures::{Future, Stream};

use salt::{Context, Salt};

// TODO: Accept some command line arguments to configure this proxy
//       Most importantly the destination address

fn main() {
    Salt::new(|c: Context| {
        let client = hyper::Client::new(&c);
        let request = client.get("http://www.google.com".parse().unwrap());

        request.and_then(|response| {
            response
                .body()
                .fold(c, |c, chunk| io::write_all(c, chunk).map(|(c, _)| c))
        })
    }).listen("localhost:7878")
        .unwrap();
}
