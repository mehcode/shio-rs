use std::sync::Arc;
use std::net::ToSocketAddrs;

use futures::{Future, Stream};
use hyper;
use hyper::server::Http;
use tokio_core::net::TcpListener;
use tokio_core::reactor::{Core, Handle};

use context::Context;
use router::Router;
use route::Route;
use handler::Handler;

pub struct Salt {
    router: Router,
}

impl Salt {
    pub fn new() -> Self {
        Salt {
            router: Default::default(),
        }
    }

    pub fn add<R: Into<Route>>(&mut self, route: R) {
        self.router.add(route.into());
    }

    pub fn run<A: ToSocketAddrs>(&mut self, addr: A) {
        // FIXME: Bind to _all_ addresses asked for

        let addr0 = addr.to_socket_addrs().unwrap().collect::<Vec<_>>()[0];

        let mut core = Core::new().unwrap();
        let handle = core.handle();

        let service = Service {
            router: Arc::new(self.router.clone()),
            handle: handle.clone(),
        };

        let protocol = Http::new();
        let listener = TcpListener::bind(&addr0, &handle).unwrap();
        let srv = listener.incoming().for_each(|(socket, addr)| {
            protocol.bind_connection(&handle, socket, addr, service.clone());

            Ok(())
        });

        core.run(srv).unwrap();
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
