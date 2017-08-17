extern crate hyper;
extern crate futures;
extern crate tokio_core;
extern crate regex;
extern crate num_cpus;
extern crate net2;

mod context;
mod handler;
mod route;
mod router;
mod salt;
mod response;
mod stack;

pub use response::{Response, FutureResponse, BoxFutureResponse};
pub use salt::Salt;
pub use context::Context;
pub use route::Route;
pub use router::Router;
pub use handler::{Handler, BoxHandler};
pub use hyper::{Method, StatusCode, header};
pub use stack::{Stack, StackHandler};

pub mod prelude {
    pub use super::{
        Salt,
        Context,
        Response,
        FutureResponse,
        Method,
        StatusCode,
        header,
    };

    pub use futures::Future;
}
