use futures::Future;
use hyper;

use super::Context;

pub trait Handler: Send + Sync {
    type Future: Future<Item = hyper::Response>;

    fn call(&self, context: Context) -> Self::Future;
}

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
