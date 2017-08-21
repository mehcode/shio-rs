extern crate hyper;
extern crate futures;
extern crate tokio_core;
extern crate regex;
extern crate num_cpus;
extern crate net2;

mod context;
mod handler;
mod salt;
mod response;
mod stack;
mod responder;
pub mod errors;
pub mod router;

pub use response::{BoxFutureResponse, FutureResponse, Response};
pub use salt::Salt;
pub use context::Context;
pub use router::Router;
pub use handler::{BoxHandler, Handler};
pub use hyper::{header, Method, StatusCode as Status};
pub use responder::Responder;
pub use stack::{Stack, StackHandler};

pub mod prelude {
    pub use super::{header, Context, FutureResponse, Method, Response, Salt, Status};

    pub use futures::Future;
}
