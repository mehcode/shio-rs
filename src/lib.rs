extern crate hyper;
extern crate futures;
extern crate tokio_core;
extern crate regex;
extern crate num_cpus;
extern crate net2;

pub mod context;
pub mod handler;
pub mod route;
pub mod router;
pub mod salt;
pub mod response;

pub use response::{Response, FutureResponse};
pub use salt::Salt;
pub use context::Context;
pub use route::Route;
pub use handler::Handler;
pub use hyper::{Method, StatusCode};

pub mod prelude {
    pub use super::{
        Salt,
        Context,
        Response,
        FutureResponse,
        Method,
        StatusCode,
    };

    pub use futures::Future;
}
