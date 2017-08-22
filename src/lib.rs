extern crate hyper;
extern crate futures;
extern crate tokio_core;
extern crate tokio_io;
extern crate bytes;
extern crate regex;
extern crate num_cpus;
extern crate net2;

mod context;
mod handler;
mod shio;
mod response;
mod stack;
mod responder;
mod service;
pub mod util;
pub mod errors;
pub mod router;

pub use response::{BoxFutureResponse, Response};
pub use shio::Shio;
pub use context::Context;
pub use handler::{BoxHandler, Handler};
pub use hyper::{header, Method, StatusCode as Status};
pub use responder::Responder;
pub use stack::{Stack, StackHandler};

pub mod prelude {
    pub use super::{header, Context, Method, Response, Shio, Status};

    pub use futures::Future;
}
