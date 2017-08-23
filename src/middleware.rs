use handler::BoxHandler;

pub trait Middleware: Send + Sync {
    fn call(&self, next: BoxHandler) -> BoxHandler;

    #[inline]
    fn boxed(self) -> BoxMiddleware
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

pub type BoxMiddleware = Box<Middleware>;
