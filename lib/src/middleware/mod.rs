use handler::BoxHandlerMut;

pub trait Middleware: Send + Sync {
    fn call(&self, next: BoxHandlerMut) -> BoxHandlerMut;

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
    TFn: Fn(BoxHandlerMut) -> BoxHandlerMut,
{
    #[inline]
    fn call(&self, next: BoxHandlerMut) -> BoxHandlerMut {
        (*self)(next)
    }
}

#[cfg_attr(feature = "cargo-clippy", allow(stutter))]
pub type BoxMiddleware = Box<Middleware>;

mod recover;
pub use self::recover::Recover;
