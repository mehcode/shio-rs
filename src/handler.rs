use futures::{Future, IntoFuture};
use hyper::StatusCode;

use response::{BoxFutureResponse, Response};
use context::Context;

pub trait Handler: Send + Sync {
    type Result: IntoFuture<Item = Response>;

    fn call(&self, context: Context) -> Self::Result;

    fn boxed(self) -> BoxHandler
    where
        Self: Sized + 'static,
    {
        Box::new(move |ctx: Context| {
            Box::new(self.call(ctx).into_future().or_else(|_| {
                // FIXME: Do something with the error argument. Perhaps require at least `:Debug`
                //        so we can let someone know they hit the default error catcher

                Response::new().with_status(StatusCode::InternalServerError)
            })) as BoxFutureResponse
        })
    }
}

pub type BoxHandler = Box<Handler<Result = BoxFutureResponse>>;

impl<TError, TFuture, TFn> Handler for TFn
where
    TFuture: IntoFuture<Item = Response, Error = TError>,
    TFn: Send + Sync,
    TFn: Fn(Context) -> TFuture,
{
    type Result = TFuture;

    #[inline]
    fn call(&self, context: Context) -> Self::Result {
        (*self)(context)
    }
}
