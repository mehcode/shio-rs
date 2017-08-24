#![cfg_attr(feature = "cargo-clippy", warn(clippy, clippy_pedantic))]
#![cfg_attr(feature = "cargo-clippy", allow(missing_docs_in_private_items))]

extern crate hyper;
extern crate futures;
extern crate tokio_core;
extern crate tokio_io;
extern crate bytes;
extern crate regex;
extern crate num_cpus;
extern crate net2;
#[macro_use]
extern crate log;

mod context;
mod handler;
mod shio;
mod stack;
mod responder;
mod service;
pub mod ext;
pub mod middleware;
pub mod response;
pub mod errors;
pub mod router;

pub use hyper::{header, Method, StatusCode};

pub use response::{BoxFutureResponse, Response};
pub use shio::Shio;
pub use context::Context;
pub use handler::{BoxHandler, Handler};
pub use responder::Responder;
pub use middleware::Middleware;
pub use stack::Stack;

/// Re-exports important traits and types. Meant to be glob imported when using Shio.
pub mod prelude {
    pub use super::{header, BoxFutureResponse, BoxHandler, Context, Handler, Method, Response,
                    Shio, StatusCode};

    pub use futures::Future;
    pub use ext::{BoxFuture, FutureExt};
}
