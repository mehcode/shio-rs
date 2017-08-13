use futures::Future;
use context::Context;

pub trait Service {
    type Error: ::std::error::Error;
    type Future: Future<Error = Self::Error>;

    fn call(&self, c: Context) -> Self::Future;
}

impl<F, E, Func> Service for Func
where
    E: ::std::error::Error,
    F: Future<Error = E>,
    Func: Fn(Context) -> F,
{
    type Error = E;
    type Future = F;

    fn call(&self, c: Context) -> Self::Future {
        (*self)(c)
    }
}
