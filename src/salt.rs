use std::sync::Arc;
use std::net::{SocketAddr, ToSocketAddrs};
use std::thread;
use std::io;

use num_cpus;
use futures::{future, Future, Stream};
use hyper::{self, StatusCode};
use hyper::server::Http;
use tokio_core::net::TcpListener;
use tokio_core::reactor::{Core, Handle};
use net2::TcpBuilder;

#[cfg(unix)]
use net2::unix::UnixTcpBuilderExt;

use handler::Handler;
use context::Context;
use router::{Router, Route};
use response::Response;
use errors::ListenError;

pub struct Salt<H: Handler + 'static> {
    handler: Arc<H>,
    threads: usize,
}

impl<H: Handler> Salt<H> {
    pub fn new(handler: H) -> Self {
        Salt {
            handler: Arc::new(handler),
            threads: num_cpus::get(),
        }
    }

    /// Set the number of threads to use.
    pub fn threads(&mut self, threads: usize) {
        self.threads = threads;
    }

    pub fn run<A: ToSocketAddrsExt>(&self, addr: A) -> Result<(), ListenError> {
        let addrs = addr.to_socket_addrs_ext()?.collect::<Vec<_>>();
        let mut children = Vec::new();

        for _ in 0..self.threads {
            let addrs = addrs.clone();
            let handler = self.handler.clone();

            children.push(thread::spawn(move || -> Result<(), ListenError> {
                let mut core = Core::new()?;
                let mut work = Vec::new();
                let handle = core.handle();

                let service = Service {
                    handler: handler,
                    handle: handle.clone(),
                };

                for addr in &addrs {
                    let handle = handle.clone();
                    let builder = (match *addr {
                        SocketAddr::V4(_) => TcpBuilder::new_v4(),
                        SocketAddr::V6(_) => TcpBuilder::new_v6(),
                    })?;

                    // Set SO_REUSEADDR on the socket
                    builder.reuse_address(true)?;

                    // Set SO_REUSEPORT on the socket (in unix)
                    #[cfg(unix)]
                    builder.reuse_port(true)?;

                    builder.bind(&addr)?;

                    let listener = TcpListener::from_listener(
                        // TODO: Should this be configurable somewhere?
                        builder.listen(128)?,
                        &addr,
                        &handle,
                    )?;

                    let protocol = Http::new();
                    let service = service.clone();

                    let srv = listener.incoming().for_each(move |(socket, addr)| {
                        protocol.bind_connection(&handle, socket, addr, service.clone());

                        Ok(())
                    });

                    work.push(srv);
                }

                core.run(future::join_all(work))?;

                Ok(())
            }));
        }

        for child in children.drain(..) {
            child.join().unwrap()?;
        }

        Ok(())
    }
}

impl Default for Salt<Router> {
    fn default() -> Self {
        Salt::new(Router::new())
    }
}

impl Salt<Router> {
    pub fn route<R: Into<Route>>(&mut self, route: R) -> &mut Self {
        Arc::get_mut(&mut self.handler).map(|router| {
            router.route(route)
        });

        self
    }
}

// FIXME: Why does #[derive(Clone)] not work here? This _seems_ like a implementation that
//        should be auto-derived.

// #[derive(Clone)]
struct Service<H: Handler + 'static> {
    handler: Arc<H>,
    handle: Handle,
}

impl<H: Handler + 'static> Clone for Service<H> {
    fn clone(&self) -> Self {
        Service { handler: self.handler.clone(), handle: self.handle.clone() }
    }
}

impl<H: Handler + 'static> hyper::server::Service for Service<H> {
    type Request = hyper::Request;
    type Response = hyper::Response;
    type Error = hyper::Error;
    type Future = Box<Future<Item = Self::Response, Error = Self::Error>>;

    fn call(&self, request: Self::Request) -> Self::Future {
        let ctx = Context::new(request, self.handle.clone());

        Box::new(self.handler.call(ctx).or_else(|_| {
            // FIXME: Do something with the error argument. Perhaps require at least `:Debug`
            //        so we can let someone know they hit the default error catcher

            Response::new().status(StatusCode::InternalServerError)
        }))
    }
}

/// An extension of [`ToSocketAddrs`] that allows for a default address when specifying just
/// the port as `:8080`.
pub trait ToSocketAddrsExt {
    type Iter: Iterator<Item = SocketAddr>;

    fn to_socket_addrs_ext(&self) -> io::Result<Self::Iter>;
}

impl<'a> ToSocketAddrsExt for &'a str {
    type Iter = <str as ToSocketAddrs>::Iter;

    fn to_socket_addrs_ext(&self) -> io::Result<Self::Iter> {
        if self.starts_with(':') {
            // If we start with `:`; assume the ip is ommitted and this is just a port
            // specification
            let port: u16 = self[1..].parse().unwrap();
            Ok((&[
                SocketAddr::new("0.0.0.0".parse().unwrap(), port),
                SocketAddr::new("::0".parse().unwrap(), port),
            ][..]).to_socket_addrs()?.collect::<Vec<_>>().into_iter())
        } else {
            self.to_socket_addrs()
        }
    }
}

impl ToSocketAddrsExt for String {
    type Iter = <String as ToSocketAddrs>::Iter;

    fn to_socket_addrs_ext(&self) -> io::Result<Self::Iter> {
        (&**self).to_socket_addrs_ext()
    }
}

macro_rules! forward_to_socket_addrs {
    ($lifetime:tt, $ty:ty) => (
        impl<$lifetime> ToSocketAddrsExt for $ty {
            type Iter = <$ty as ToSocketAddrs>::Iter;

            fn to_socket_addrs_ext(&self) -> io::Result<Self::Iter> {
                self.to_socket_addrs()
            }
        }
    );

    ($ty:ty) => (
        impl ToSocketAddrsExt for $ty {
            type Iter = <$ty as ToSocketAddrs>::Iter;

            fn to_socket_addrs_ext(&self) -> io::Result<Self::Iter> {
                self.to_socket_addrs()
            }
        }
    );
}

forward_to_socket_addrs!('a, &'a [SocketAddr]);
forward_to_socket_addrs!('a, (&'a str, u16));

#[cfg(test)]
mod tests {
    use super::ToSocketAddrsExt;

    #[test]
    fn to_socket_addrs_ext_str() {
        let addresses = ":7878".to_socket_addrs_ext().unwrap().collect::<Vec<_>>();

        assert_eq!(addresses.len(), 2);
        assert_eq!(addresses[0], "0.0.0.0:7878".parse().unwrap());
        assert_eq!(addresses[1], "[::0]:7878".parse().unwrap());
    }

    #[test]
    fn to_socket_addrs_ext_string() {
        let address = ":7878".to_owned();
        let addresses = address.to_socket_addrs_ext().unwrap().collect::<Vec<_>>();

        assert_eq!(addresses.len(), 2);
        assert_eq!(addresses[0], "0.0.0.0:7878".parse().unwrap());
        assert_eq!(addresses[1], "[::0]:7878".parse().unwrap());
    }
}
