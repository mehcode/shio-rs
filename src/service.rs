use tokio_core::net::TcpStream;
use futures::Future;

pub trait Service {
    type Error: ::std::error::Error;
    type Future: Future<Error = Self::Error>;

    fn call(&self, socket: TcpStream) -> Self::Future;
}

impl<F, E, Func> Service for Func
where
    E: ::std::error::Error,
    F: Future<Error = E>,
    Func: Fn(TcpStream) -> F,
{
    type Error = E;
    type Future = F;

    fn call(&self, socket: TcpStream) -> Self::Future {
        (*self)(socket)
    }
}
