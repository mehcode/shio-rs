mod body;

use hyper::{self, Method};

pub use self::body::Body;

pub struct Request {
    method: Method,
    uri: hyper::Uri,
    version: hyper::HttpVersion,
    headers: hyper::Headers,
    body: Body,
}

impl Request {
    pub(crate) fn new(
        components: (
            Method,
            hyper::Uri,
            hyper::HttpVersion,
            hyper::Headers,
            hyper::Body,
        ),
    ) -> Self {
        Self {
            method: components.0,
            uri: components.1,
            version: components.2,
            headers: components.3,
            body: Body::new(components.4),
        }
    }

    /// Returns a reference to the request HTTP version.
    #[inline]
    pub fn version(&self) -> &hyper::HttpVersion {
        &self.version
    }

    /// Returns a reference to the request headers.
    #[inline]
    pub fn headers(&self) -> &hyper::Headers {
        &self.headers
    }

    /// Returns a reference to the request HTTP method.
    #[inline]
    pub fn method(&self) -> &Method {
        &self.method
    }

    /// Returns a reference to the request URI.
    #[inline]
    pub fn uri(&self) -> &hyper::Uri {
        &self.uri
    }

    /// Returns a reference to the request path.
    #[inline]
    pub fn path(&self) -> &str {
        self.uri.path()
    }

    /// Take the request body.
    #[inline]
    pub fn body(self) -> Body {
        self.body
    }
}
