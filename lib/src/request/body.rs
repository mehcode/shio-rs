use hyper;
use futures::{Poll, Stream};

use errors::Error;

#[derive(Default)]
pub struct Body(hyper::Body);

impl Body {
    pub(crate) fn new(body: hyper::Body) -> Self {
        Body(body)
    }
}

impl Stream for Body {
    type Item = hyper::Chunk;
    type Error = Error;

    #[inline]
    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        self.0.poll().map_err(Error::from)
    }
}
