mod route;
mod pattern;

use std::collections::HashMap;

use hyper::{Method, StatusCode};
use regex::RegexSet;
use futures::future;

use handler::Handler;
use context::Context;
use response::{BoxFutureResponse, Response};
pub use router::route::Route;
pub use router::pattern::Pattern;

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
    /// # use shio::{Method, Response, Status};
    /// # use shio::router::Router;
    /// # let mut router = Router::new();
    /// router.route((Method::Get, "/users", |_| {
    ///     // [...]
    /// # Response::with(Status::NoContent)
    /// }));
    /// ```
    ///
    /// [`Method`]: https://docs.rs/hyper/0.11/hyper/enum.Method.html
    /// [`Pattern`]: struct.Pattern.html
    pub fn route<R: Into<Route>>(&mut self, route: R) {
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

    pub(crate) fn find(&self, method: &Method, path: &str) -> Option<&Route> {
        // Pull out the patterns and routes for this method
        let routes = try_opt!(self.routes.get(method));
        let route_patterns = try_opt!(self.route_patterns.get(method));

        // Get the first match and return it
        let matched_index = try_opt!(route_patterns.matches(path).into_iter().next());
        Some(&routes[matched_index])
    }
}

impl Handler for Router {
    type Result = BoxFutureResponse;

    #[inline]
    fn call(&self, ctx: Context) -> Self::Result {
        let route = match self.find(ctx.method(), ctx.path()) {
            Some(route) => route,
            None => {
                // Return 404 if we failed to find a matching route
                return Box::new(future::ok(Response::with(StatusCode::NotFound)));
            }
        };

        route.call(ctx)
    }
}
