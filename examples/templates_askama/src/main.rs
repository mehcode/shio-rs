extern crate shio;

#[macro_use]
extern crate askama;

use shio::prelude::*;
use askama::Template;

#[derive(Template)]
#[template(path = "hello.html")]
struct HelloTemplate {
    name: String,
}

fn hello(ctx: Context) -> HelloTemplate {
    HelloTemplate {
        name: ctx.get::<Parameters>()["name"].into(),
    }
}

fn main() {
    Shio::default()
        .route((Method::Get, "/hello/{name}", hello))
        .run(":7878")
        .unwrap()
}
