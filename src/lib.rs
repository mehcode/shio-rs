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
mod middleware;
pub mod response;
pub mod util;
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

pub mod prelude {
    //! Re-exports important traits and types. Meant to be glob imported when using Shio.

    pub use super::{header, BoxFutureResponse, BoxHandler, Context, Handler, Method, Response,
                    Shio, StatusCode};

    pub use futures::Future;
}
