use handler::BoxHandler;

pub trait Middleware: Send + Sync {
    fn call(&self, next: BoxHandler) -> BoxHandler;

    #[inline]
    fn into_box(self) -> BoxMiddleware
    where
        Self: Sized + 'static,
    {
        Box::new(self)
    }
}

impl<TFn> Middleware for TFn
where
    TFn: Send + Sync,
    TFn: Fn(BoxHandler) -> BoxHandler,
{
    #[inline]
    fn call(&self, next: BoxHandler) -> BoxHandler {
        (*self)(next)
    }
}

#[cfg_attr(feature = "cargo-clippy", allow(stutter))]
pub type BoxMiddleware = Box<Middleware>;


mod recover;

/// Middleware that catches `panic!`, returning an error 500 to the user.
pub struct Recover;

impl Middleware for Recover {
    fn call(&self, next: BoxHandler) -> BoxHandler {
        recover::recover_panics(next)
    }
}