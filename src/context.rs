use std::ops::Deref;
use std::io::{self, Read, Write};

use hyper::{self, Method, Request, HttpVersion, Uri, Headers, Body};
use tokio_core::reactor::Handle;
use tokio_io::AsyncRead;
use futures::{Poll, Async, Stream};
use bytes::{BufMut};

/// `Context` represents the context of the current HTTP request.
///
/// A `Context` consists of:
///     - A [`Handle`] referencing the event loop in which this request is being
///       handled.
///     - The current HTTP [`Request`].
///
/// [`Handle`]: https://docs.rs/tokio-core/0.1/tokio_core/reactor/struct.Handle.html
/// [`Request`]: http://doc.rust-lang.org/hyper/0.11/hyper/client/struct.Request.html
pub struct Context {
    method: Method,
    uri: Uri,
    version: HttpVersion,
    headers: Headers,
    body: Body,
    handle: Handle,
}

impl Context {
    pub(crate) fn new(request: Request, handle: Handle) -> Self {
        let (method, uri, version, headers, body) = request.deconstruct();

        Context { handle,
            method,
            uri,
            version,
            headers,
            body,
        }
    }

    /// Return a reference to a handle to the event loop this `Context` is associated with.
    #[inline]
    pub fn handle(&self) -> &Handle {
        &self.handle
    }

    /// Returns a reference to the request HTTP version.
    #[inline]
    pub fn version(&self) -> &HttpVersion {
        &self.version
    }

    /// Returns a reference to the request headers.
    #[inline]
    pub fn headers(&self) -> &Headers {
        &self.headers
    }

    /// Returns a reference to the request HTTP method.
    #[inline]
    pub fn method(&self) -> &Method {
        &self.method
    }

    /// Returns a reference to the request URI.
    #[inline]
    pub fn uri(&self) -> &Uri {
        &self.uri
    }

    /// Returns a reference to the request path.
    #[inline]
    pub fn path(&self) -> &str {
        self.uri.path()
    }

    /// Returns a reference to the request body.
    #[inline]
    pub fn body(&self) -> &Body {
        &self.body
    }
}

impl Deref for Context {
    type Target = Handle;

    fn deref(&self) -> &Self::Target {
        &self.handle
    }
}

impl Read for Context {
    fn read(&mut self, mut buf: &mut [u8]) -> io::Result<usize> {
        match self.body.poll() {
            Ok(Async::Ready(chunk)) => {
                Ok(match chunk {
                    Some(mut chunk) => {
                        buf.write_all(&mut chunk)?;
                        chunk.len()
                    }

                    None => {
                        0
                    }
                })
            }

            Ok(Async::NotReady) => Err(io::ErrorKind::WouldBlock.into()),
            Err(error) => {
                match error {
                    hyper::Error::Io(error) => Err(error),
                    _ => {
                        Err(io::Error::new(io::ErrorKind::Other, Box::new(error)))
                    }
                }
            }
        }
    }
}

impl AsyncRead for Context {
    fn read_buf<B: BufMut>(&mut self, buf: &mut B) -> Poll<usize, io::Error> {
        match self.body.poll() {
            Ok(Async::Ready(chunk)) => {
                Ok(Async::Ready(match chunk {
                    Some(mut chunk) => {
                        buf.put_slice(&mut chunk);
                        chunk.len()
                    }

                    None => {
                        0
                    }
                }))
            }

            Ok(Async::NotReady) => Ok(Async::NotReady),
            Err(error) => {
                match error {
                    hyper::Error::Io(error) => Err(error),
                    _ => {
                        Err(io::Error::new(io::ErrorKind::Other, Box::new(error)))
                    }
                }
            }
        }
    }
}
