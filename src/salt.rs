use std::sync::Arc;
use std::net::{SocketAddr, ToSocketAddrs};
use std::thread;

use num_cpus;
use futures::{Future, Stream};
use hyper;
use hyper::server::Http;
use tokio_core::net::TcpListener;
use tokio_core::reactor::{Core, Handle};
use net2::TcpBuilder;

#[cfg(unix)]
use net2::unix::UnixTcpBuilderExt;

use super::{Route, Handler, Context, Router, StatusCode, Response};

pub struct Salt<H: Handler + 'static> {
    handler: H,
    threads: usize,
}

impl<H: Handler> Salt<H> {
    pub fn new(handler: H) -> Self {
        Salt {
            handler: handler,
            threads: num_cpus::get(),
        }
    }

    /// Set the number of threads to use.
    pub fn threads(&mut self, threads: usize) {
        self.threads = threads;
    }

    pub fn run<A: ToSocketAddrs>(self, addr: A) {
        // FIXME: Bind to _all_ addresses asked for
        // FIXME: Return a Result on failure instead of all these unwraps

        let addr0 = addr.to_socket_addrs().unwrap().collect::<Vec<_>>()[0];
        // let router = Arc::new(self.router.clone());
        let mut children = Vec::new();
        let handler = Arc::new(self.handler);

        for _ in 0..self.threads {
            let handler = handler.clone();

            children.push(thread::spawn(move || {
                let mut core = Core::new().unwrap();
                let handle = core.handle();

                let service = Service {
                    handler: handler,
                    handle: handle.clone(),
                };

                let builder = (match addr0 {
                    SocketAddr::V4(_) => TcpBuilder::new_v4(),
                    SocketAddr::V6(_) => TcpBuilder::new_v6(),
                }).unwrap();

                // Set SO_REUSEADDR on the socket
                builder.reuse_address(true).unwrap();

                // Set SO_REUSEPORT on the socket (in unix)
                #[cfg(unix)]
                builder.reuse_port(true).unwrap();

                builder.bind(&addr0).unwrap();

                let listener = TcpListener::from_listener(
                    builder.listen(128).unwrap(),
                    &addr0,
                    &core.handle(),
                ).unwrap();

                let protocol = Http::new();
                let srv = listener.incoming().for_each(|(socket, addr)| {
                    protocol.bind_connection(&handle, socket, addr, service.clone());

                    Ok(())
                });

                core.run(srv).unwrap();
            }));
        }

        for child in children.drain(..) {
            let _ = child.join();
        }
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

struct Service<H: Handler + 'static> {
    handler: Arc<H>,
    handle: Handle,
}

impl<H: Handler + 'static> Service<H> {
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
