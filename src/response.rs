use futures::{Poll, Async, Future};
use hyper;

// TODO: Look into simplifying this. I don't like the .take in the Future impl
//       and the builder impls are very verbose. Should probably use the type system more and
//       have the compiler yell if someone tries to set the body twice (for instance)

pub struct Response {
    inner: Option<hyper::Response<hyper::Body>>,
}

impl Response {
    pub fn new() -> Response {
        Response { inner: Some(hyper::Response::new()) }
    }

    pub fn body<B: Into<hyper::Body>>(mut self, body: B) -> Self {
        match self.inner {
            Some(ref mut response) => {
                response.set_body(body.into());
            }

            None => {
                // Do nothing; this response has already been returned to the client
                // TODO: Perhaps raise an error?
            }
        }

        self
    }
}

impl Future for Response {
    type Item = hyper::Response<hyper::Body>;
    type Error = hyper::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let raw_response = self.inner.take().unwrap();

        Ok(Async::Ready(raw_response))
    }
}
