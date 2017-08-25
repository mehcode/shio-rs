mod route;
mod pattern;

pub use router::route::Route;
pub use router::pattern::Pattern;

use std::collections::HashMap;

use hyper::{self, Method, StatusCode};
use regex::RegexSet;
use futures::future;

use handler::Handler;
use context::Context;
use response::Response;
use ext::BoxFuture;

// From: https://github.com/crumblingstatue/try_opt/blob/master/src/lib.rs#L30
macro_rules! try_opt {
    ($e:expr) =>(
        match $e {
            Some(v) => v,
            None => return None,
        }
    )
}

#[derive(Default)]
pub struct Router {
    routes: HashMap<Method, Vec<Route>>,
    route_patterns: HashMap<Method, RegexSet>,
}

impl Router {
    /// Construct a new, empty `Router`.
    ///
    /// ```rust
    /// # use shio::router::Router;
    /// let router = Router::new();
    /// ```
    ///
    /// Equivalent to `Router::default()`.
    pub fn new() -> Self {
        Default::default()
    }

    /// Add a new route to a `Router`, matching both a [`Method`] and a [`Pattern`]. Any value
    /// that implements `Into<Route>` may be provided to `route`.
    ///
    /// For example, to match a `Get` request to `/users`:
    ///
    /// ```rust
    /// # use shio::{Method, Response, StatusCode};
    /// # use shio::router::Router;
    /// # let mut router = Router::new();
    /// router.route((Method::Get, "/users", |_| {
    ///     // [...]
    /// # Response::with(StatusCode::NoContent)
    /// }));
    /// ```
    ///
    /// [`Method`]: https://docs.rs/hyper/0.11/hyper/enum.Method.html
    /// [`Pattern`]: struct.Pattern.html
    pub fn add<R: Into<Route>>(&mut self, route: R) {
        let route: Route = route.into();
        let method = route.method().clone();

        self.routes
            .entry(method.clone())
            .or_insert_with(Vec::new)
            .push(route);

        let routes = &self.routes[&method];

        // FIXME: Think of some way to not re-compile method maps so often during server boot
        // NOTE: The .unwrap cannot fail as we are using route patterns that are pre-verified
        self.route_patterns.insert(
            method.clone(),
            RegexSet::new(routes.iter().map(|r| r.pattern())).unwrap(),
        );
    }

    #[deprecated(since = "0.0.7", note = "use `Router::add` instead")]
    pub fn route<R: Into<Route>>(&mut self, route: R) {
        self.add(route);
    }

    pub(crate) fn find(&self, method: &Method, uri: &str) -> Option<&Route> {
        // Pull out the patterns and routes for this method
        let routes = try_opt!(self.routes.get(method));
        let route_patterns = try_opt!(self.route_patterns.get(method));

        // Get the first match and return it
        let matched_index = try_opt!(route_patterns.matches(uri).into_iter().next());
        Some(&routes[matched_index])
    }
}

impl Handler for Router {
    type Result = BoxFuture<Response, hyper::Error>;

    #[inline]
    fn call(&self, ctx: Context) -> Self::Result {
        if let Some(route) = self.find(ctx.method(), ctx.path()) {
            route.call(ctx)
        } else {
            // Return 404 if we failed to find a matching route
            Box::new(future::ok(Response::with(StatusCode::NotFound)))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Router, Route};
    use Method::*;
    use Context;

    // Empty handler to use for route tests
    fn empty_handler(_: Context) {
    }

    // Construct a router with the provided routes
    fn router(routes: &[&'static str]) -> Router {
        let mut router = Router::new();
        for route in routes {
            let route = Route::from((Get, *route, empty_handler));
            router.add(route);
        }

        router
    }

    #[test]
    fn test_static() {
        let mut router = Router::new();
        router.add((Get, "/hello", empty_handler));

        assert!(router.find(&Get, "/hello").is_some());
        assert!(router.find(&Get, "/aa").is_none());
        assert!(router.find(&Get, "/hello/asfa").is_none());

        let mut router = Router::new();
        router.add((Put, "/hello", empty_handler));
        router.add((Post, "/hello", empty_handler));
        router.add((Delete, "/hello", empty_handler));

        assert!(router.find(&Get, "/hello").is_none());
        assert!(router.find(&Put, "/hello").is_some());
        assert!(router.find(&Post, "/hello").is_some());
        assert!(router.find(&Delete, "/hello").is_some());
    }
}
