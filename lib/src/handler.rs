use std::fmt;
use std::marker::PhantomData;

use futures::Future;
use hyper;

use response::Response;
use context::Context;
use StatusCode;
use ext::{BoxFuture, FutureExt, IntoFutureExt};

#[cfg_attr(feature = "cargo-clippy", allow(needless_pass_by_value, use_debug))]
pub(crate) fn default_catch<'r, E: fmt::Debug + Send + 'r>(err: E) -> Response {
    // TODO: Support definable error catchers
    /*
        Idea of syntax:

        Shio::new().catch(|err: Error| { ... })
    */

    // Default "error catcher" just logs with error! and responds with a 500
    error!("{:?}", err);

    Response::build()
        .status(StatusCode::InternalServerError)
        .into()
}

pub trait HandlerMut<'r>: Send + Sync
where
    <Self::Result as IntoFutureExt<'r, Response>>::Error: fmt::Debug + Send + Sync + 'r,
{
    type Result: IntoFutureExt<'r, Response> + 'r;

    fn call(&self, context: &'r mut Context) -> Self::Result;
}

pub(crate) struct BoxedHandlerMut<'r, H>
where
    H: HandlerMut<'r> + 'r,
    <H::Result as IntoFutureExt<'r, Response>>::Error: fmt::Debug + Send + Sync + 'r,
{
    handler: H,
    phantom: PhantomData<&'r H>,
}

impl<'r, H> BoxedHandlerMut<'r, H>
where
    H: HandlerMut<'r> + 'r,
    <H::Result as IntoFutureExt<'r, Response>>::Error: fmt::Debug + Send + Sync + 'r,
{
    fn new(handler: H) -> Self {
        BoxedHandlerMut {
            handler,
            phantom: PhantomData,
        }
    }
}

impl<'r, H: HandlerMut<'r>> HandlerMut<'r> for BoxedHandlerMut<'r, H>
where
    <H::Result as IntoFutureExt<'r, Response>>::Error: fmt::Debug + Send + Sync + 'r,
{
    type Result = BoxFuture<'r, Response, hyper::Error>;

    #[inline]
    fn call(&self, context: &'r mut Context) -> Self::Result {
        Box::new(
            self.handler
                .call(context)
                .into_future_ext()
                .or_else(default_catch),
        )
    }
}

#[cfg_attr(feature = "cargo-clippy", allow(stutter))]
pub type BoxHandlerMut<'r> = Box<
    HandlerMut<'r, Result = BoxFuture<'r, Response, hyper::Error>> + 'r,
>;

pub trait Handler<'r>: Send + Sync
where
    <Self::Result as IntoFutureExt<'r, Response>>::Error: fmt::Debug + Send + Sync + 'r,
{
    type Result: IntoFutureExt<'r, Response> + 'r;

    fn call(&self, context: &'r Context) -> Self::Result;

    #[inline]
    fn into_box(self) -> BoxHandler<'r>
    where
        Self: Sized + 'r,
    {
        Box::new(BoxedHandler::new(self))
    }
}

pub(crate) struct BoxedHandler<'r, H>
where
    H: HandlerMut<'r> + 'r,
    <H::Result as IntoFutureExt<'r, Response>>::Error: fmt::Debug + Send + Sync + 'r,
{
    handler: H,
    phantom: PhantomData<&'r H>,
}

impl<'r, H> BoxedHandler<'r, H>
where
    H: Handler<'r> + 'r,
    <H::Result as IntoFutureExt<'r, Response>>::Error: fmt::Debug + Send + Sync + 'r,
{
    fn new(handler: H) -> Self {
        BoxedHandler {
            handler,
            phantom: PhantomData,
        }
    }
}

impl<'r, H: Handler<'r>> Handler<'r> for BoxedHandler<'r, H>
where
    <H::Result as IntoFutureExt<'r, Response>>::Error: fmt::Debug + Send + Sync + 'r,
{
    type Result = BoxFuture<'r, Response, hyper::Error>;

    #[inline]
    fn call(&self, context: &'r Context) -> Self::Result {
        Box::new(
            self.handler
                .call(context)
                .into_future_ext()
                .or_else(default_catch),
        )
    }
}

#[cfg_attr(feature = "cargo-clippy", allow(stutter))]
pub type BoxHandler<'r> = Box<Handler<'r, Result = BoxFuture<'r, Response, hyper::Error>> + 'r>;

impl<'r, TError, TFuture, TFn> Handler<'r> for TFn
where
    TError: fmt::Debug + Send + Sync + 'r,
    TFuture: IntoFutureExt<'r, Response, Error = TError> + 'r,
    TFn: Send + Sync,
    TFn: Fn(&'r Context) -> TFuture,
{
    type Result = TFuture;

    #[inline]
    fn call(&self, context: &'r Context) -> Self::Result {
        self(context)
    }
}

impl<'r, H: Handler<'r>> HandlerMut<'r> for H {
    type Result = H::Result;

    #[inline]
    fn call(&self, context: &'r mut Context) -> Self::Result {
        Handler::call(self, context)
    }
}
