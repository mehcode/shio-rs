use futures::Future;

/// A type alias for Box<Item = T, Error = E>
#[cfg_attr(feature = "cargo-clippy", allow(stutter))]
pub type BoxFuture<T, E> = Box<Future<Item = T, Error = E>>;

pub trait FutureExt: Future {
    fn into_box(self) -> BoxFuture<Self::Item, Self::Error>
    where
        Self: Sized + 'static,
    {
        Box::new(self)
    }
}

impl<F: Future> FutureExt for F {}
