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

pub use response::{Response, FutureResponse, BoxFutureResponse};
pub use salt::Salt;
pub use context::Context;
pub use router::Router;
pub use handler::{Handler, BoxHandler};
pub use hyper::{Method, StatusCode as Status, header};
pub use responder::Responder;
pub use stack::{Stack, StackHandler};

pub mod prelude {
    pub use super::{
        Salt,
        Context,
        Response,
        FutureResponse,
        Method,
        Status,
        header,
    };

    pub use futures::Future;
}
