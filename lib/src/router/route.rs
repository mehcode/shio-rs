use std::fmt;

use hyper::{self, Method};

use context::Context;
use handler::{BoxHandler, Handler};
use response::Response;
use router::pattern::Pattern;
use ext::{BoxFuture, IntoFutureExt};

/// Route contains a [`Handler`] and information for matching against requests.
pub struct Route {
    method: Method,
    pattern: Pattern,
    handler: BoxHandler,
}

impl Route {
    /// Constructs a new `Route` which matches against the provided information.
    ///
    /// ```rust
    /// # use shio::{Response, Method};
    /// # use shio::router::Route;
    /// Route::new(Method::Post, "/inbox", |_| {
    ///     // [...]
    /// # Response::new()
    /// });
    /// ```
    pub fn new<P, H>(method: Method, pattern: P, handler: H) -> Self
    where
        P: Into<Pattern>,
        H: Handler + 'static,
        <H::Result as IntoFutureExt<Response>>::Error: fmt::Debug + Send + Sync,
    {
        Route {
            handler: handler.into_box(),
            pattern: pattern.into(),
            method,
        }
    }

    #[inline]
    pub(crate) fn method(&self) -> &Method {
        &self.method
    }

    #[inline]
    pub(crate) fn pattern(&self) -> &Pattern {
        &self.pattern
    }
}

impl<P, H> From<(Method, P, H)> for Route
where
    P: Into<Pattern>,
    H: Handler + 'static,
    <H::Result as IntoFutureExt<Response>>::Error: fmt::Debug + Send + Sync,
{
    fn from(arguments: (Method, P, H)) -> Self {
        Route::new(arguments.0, arguments.1, arguments.2)
    }
}

impl Handler for Route {
    type Result = BoxFuture<Response, hyper::Error>;

    #[inline]
    fn call(&self, ctx: Context) -> Self::Result {
        self.handler.call(ctx)
    }
}

impl fmt::Debug for Route {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Route {{ method: {:?}, pattern: {:?}, .. }}",
            self.method,
            self.pattern.as_str()
        )
    }
}
