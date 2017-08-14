#![feature(conservative_impl_trait)]

extern crate salt;
extern crate hyper;
extern crate futures;

use salt::{http, Context, Response, Salt};
use hyper::Client;
use futures::Future;

fn index(_: Context) -> Response {
    Response::new().body("Hello World!")
}

// FIXME: `impl Future<Item = hyper::Response>` is currently required instead of the
//        simpler `impl Future`. Planning to work more on the Response type to hide
//        `hyper::Response` as well as make the signatures a touch nicer

fn proxy_google(c: Context) -> impl Future<Item = hyper::Response> {
    let client = Client::new(&c);
    let google_url = "http://www.google.com".parse().unwrap();
    let google_request = client.get(google_url);

    google_request.and_then(move |google_response| {
        Response::new().body(google_response.body())
    })
}

fn main() {
    let mut s = Salt::new();

    s.add((http::method::GET, "/google", proxy_google));
    s.add((http::method::GET, "/", index));

    s.run("127.0.0.1:7878");
}
