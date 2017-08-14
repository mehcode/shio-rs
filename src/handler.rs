use futures::Future;
use context::Context;
use hyper::{Response, Body};

pub trait Handler {
    type Future: Future<Item = Response<Body>>;

    fn call(&self, context: Context) -> Self::Future;
}

impl<E, R, F> Handler for F
where
    R: Future<Item = Response<Body>, Error = E>,
    F: Fn(Context) -> R,
{
    type Future = R;

    fn call(&self, context: Context) -> Self::Future {
        (*self)(context)
    }
}
