use futures::Future;
use hyper::{self, StatusCode};

use response::{Response, BoxFutureResponse};
use context::Context;

pub trait Handler: Send + Sync {
    type Future: Future<Item = hyper::Response>;

    fn call(&self, context: Context) -> Self::Future;

    fn boxed(self) -> BoxHandler where Self: Sized + 'static {
        Box::new(move |ctx: Context| {
            Box::new(self.call(ctx).or_else(|_| {
                // FIXME: Do something with the error argument. Perhaps require at least `:Debug`
                //        so we can let someone know they hit the default error catcher

                Response::new().status(StatusCode::InternalServerError)
            })) as BoxFutureResponse
        })
    }
}

pub type BoxHandler = Box<Handler<Future = BoxFutureResponse>>;

impl<TError, TFuture, TFn> Handler for TFn
where
    TFuture: Future<Item = hyper::Response, Error = TError>,
    TFn: Send + Sync,
    TFn: Fn(Context) -> TFuture,
{
    type Future = TFuture;

    #[inline]
    fn call(&self, context: Context) -> Self::Future {
        (*self)(context)
    }
}
