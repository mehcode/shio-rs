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

use context::Context;
use router::Router;
use route::Route;
use handler::Handler;

pub struct Salt {
    router: Router,
    threads: usize,
}

impl Salt {
    pub fn new() -> Self {
        Salt {
            threads: num_cpus::get(),
            router: Default::default(),
        }
    }

    /// Set the number of threads to use.
    pub fn threads(&mut self, threads: usize) {
        self.threads = threads;
    }

    pub fn add<R: Into<Route>>(&mut self, route: R) {
        self.router.add(route.into());
    }

    pub fn run<A: ToSocketAddrs>(&mut self, addr: A) {
        // FIXME: Bind to _all_ addresses asked for
        // FIXME: Return a Result on failure instead of all these unwraps

        let addr0 = addr.to_socket_addrs().unwrap().collect::<Vec<_>>()[0];
        let router = Arc::new(self.router.clone());
        let mut children = Vec::new();

        for _ in 0..self.threads {
            let router = router.clone();

            children.push(thread::spawn(move || {
                let mut core = Core::new().unwrap();
                let handle = core.handle();

                let service = Service {
                    router,
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

#[derive(Clone)]
struct Service {
    router: Arc<Router>,
    handle: Handle,
}

impl hyper::server::Service for Service {
    type Request = hyper::Request;
    type Response = hyper::Response;
    type Error = hyper::Error;
    type Future = Box<Future<Item = Self::Response, Error = Self::Error>>;

    fn call(&self, request: Self::Request) -> Self::Future {
        self.router.call(Context::new(request, self.handle.clone()))
    }
}
