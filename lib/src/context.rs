use std::ops::Deref;
use std::io::{self, Read, Write};

use hyper::{self, Body, Chunk, Headers, HttpVersion, Method, Request, Uri};
use tokio_core::reactor::Handle;
use tokio_io::AsyncRead;
use futures::{Async, Stream};

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
    handle: Handle,
    body: Body,

    // Used as a buffer when reading the body through `tokio_io::AsyncRead`. This should
    // hopefully become unneccessary when `hyper::Body` internally
    // implements `tokio_io::AsyncRead`.
    chunk: Option<(Chunk, usize)>,
}

impl Context {
    pub(crate) fn new(request: Request, handle: Handle) -> Self {
        let (method, uri, version, headers, body) = request.deconstruct();

        Context {
            handle,
            method,
            uri,
            version,
            headers,
            body,
            chunk: None,
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

impl Context {
    #[inline]
    fn read_from_chunk(
        &mut self,
        chunk: Chunk,
        mut buf: &mut [u8],
        index: usize,
    ) -> io::Result<usize> {
        let written = buf.write(&chunk[index..])?;

        self.chunk = if index + written < chunk.len() {
            Some((chunk, index + written))
        } else {
            None
        };

        Ok(written)
    }
}

impl Read for Context {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if let Some((chunk, index)) = self.chunk.take() {
            return self.read_from_chunk(chunk, buf, index);
        }

        match self.body.poll() {
            Ok(Async::Ready(chunk)) => Ok(match chunk {
                Some(chunk) => self.read_from_chunk(chunk, buf, 0)?,
                None => 0,
            }),

            Ok(Async::NotReady) => Err(io::ErrorKind::WouldBlock.into()),
            Err(error) => match error {
                hyper::Error::Io(error) => Err(error),
                _ => Err(io::Error::new(io::ErrorKind::Other, Box::new(error))),
            },
        }
    }
}

impl AsyncRead for Context {}
