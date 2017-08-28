use std::fmt;

use futures::Future;

/// A type alias for Box<Item = T, Error = E>
#[cfg_attr(feature = "cargo-clippy", allow(stutter))]
pub type BoxFuture<'a, T, E> = Box<Future<Item = T, Error = E> + 'a>;

pub trait FutureExt: Future {
    fn into_box<'r>(self) -> BoxFuture<'r, Self::Item, Self::Error>
    where
        Self: Sized + 'r,
    {
        Box::new(self)
    }
}

impl<F: Future> FutureExt for F {}

pub trait IntoFutureExt<'r, T> {
    type Error: fmt::Debug + Send + Sync;
    type Future: Future<Item = T, Error = Self::Error> + 'r;

    fn into_future_ext(self) -> Self::Future;
}
