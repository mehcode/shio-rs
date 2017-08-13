extern crate futures;
extern crate tokio_core;
extern crate tokio_io;
extern crate num_cpus;
extern crate net2;

#[macro_use]
extern crate error_chain;

mod errors;

use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::net::{SocketAddr, ToSocketAddrs};
use tokio_core::reactor::Core;
use tokio_core::net::{TcpListener, TcpStream};
use futures::{future, Future, Stream};
use errors::*;
use net2::TcpBuilder;

#[cfg(unix)]
use net2::unix::UnixTcpBuilderExt;

#[derive(Default)]
pub struct Salt<
    I,
    F: Future<Item = I, Error = E> + Send,
    S: Fn(TcpStream) -> F + Sync + Send + 'static,
    E: ::std::error::Error,
> {
    /// Service that will handle incoming connections.
    service: Arc<S>,

    /// Number of threads running simultaneous event loops. Note that settings this to `0` will
    /// result in `0` event loops being run.
    ///
    /// Defaults to the number of CPUs on the machine.
    threads: usize,

    /// Collection of join handles that make up the child threads for the running server.
    children: Vec<JoinHandle<()>>,
}

impl<I, F: Future<Item = I, Error = E>, S: Fn(TcpStream) -> F, E: ::std::error::Error>
    Salt<I, F, S, E>
where
    F: Send,
    S: Sync + Send + 'static,
{
    pub fn new(service: S) -> Self {
        Salt {
            service: Arc::new(service),
            threads: num_cpus::get(),
            children: Vec::new(),
        }
    }

    pub fn close(&mut self) {
        // TODO: Signal the workers that they should stop accepting requests

        // Drain children threads and join each one
        for child in self.children.drain(..) {
            let _ = child.join();
        }
    }

    pub fn listen<A: ToSocketAddrs>(&mut self, addr: A) -> Result<()> {
        // Close the server if we are actively listening
        self.close();

        let addrs = addr.to_socket_addrs()?.collect::<Vec<_>>();

        for _ in 0..self.threads {
            let addrs = addrs.clone();
            let service = self.service.clone();

            self.children.push(thread::spawn(move || {
                let mut core = Core::new().unwrap();
                let mut work =
                    Vec::<Box<Future<Item = (), Error = Box<::std::error::Error>>>>::new();

                for addr in &addrs {
                    let builder = (match *addr {
                        SocketAddr::V4(_) => TcpBuilder::new_v4(),
                        SocketAddr::V6(_) => TcpBuilder::new_v6(),
                    }).unwrap();

                    // Set SO_REUSEADDR on the socket
                    builder.reuse_address(true).unwrap();

                    // Set SO_REUSEPORT on the socket (in unix)
                    #[cfg(unix)]
                    builder.reuse_port(true).unwrap();

                    builder.bind(&addr).unwrap();

                    let listener = TcpListener::from_listener(
                        builder.listen(128).unwrap(),
                        &addr,
                        &core.handle(),
                    ).unwrap();
                    let clients = listener.incoming();

                    work.push(Box::new(
                        clients
                            .map_err::<Box<::std::error::Error>, _>(|err| err.into())
                            .and_then(|(socket, _)| (service)(socket).from_err())
                            .into_future()
                            .map(|_| ())
                            .map_err::<_, Box<::std::error::Error>>(|(err, _)| err.into()),
                    ));
                }

                core.run(future::join_all(work)).unwrap();
            }));
        }

        Ok(())
    }
}

impl<I, F: Future<Item = I, Error = E>, S: Fn(TcpStream) -> F, E: ::std::error::Error> Drop
    for Salt<I, F, S, E>
where
    F: Send,
    S: Sync + Send + 'static,
{
    fn drop(&mut self) {
        self.close();
    }
}
