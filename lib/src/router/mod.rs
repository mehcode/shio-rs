mod route;
mod pattern;

pub use router::route::Route;
pub use router::pattern::Pattern;

use std::collections::HashMap;

use hyper::{self, Method, StatusCode};
use regex::{RegexSet, SetMatchesIntoIter};
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
            RegexSet::new(routes.iter().map(|r| r.pattern().as_str())).unwrap(),
        );
    }

    #[deprecated(since = "0.0.7", note = "use `Router::add` instead")]
    pub fn route<R: Into<Route>>(&mut self, route: R) {
        self.add(route);
    }

    pub fn find(&self, method: &Method, uri: &str) -> Option<&Route> {
        let (routes, mut route_indexes) = try_opt!(self.filter(method, uri));
        let index = try_opt!(route_indexes.next());

        Some(&routes[index])
    }

    pub fn filter(&self, method: &Method, uri: &str) -> Option<(&[Route], SetMatchesIntoIter)> {
        // Pull out the patterns and routes for this method
        let routes = try_opt!(self.routes.get(method));
        let route_patterns = try_opt!(self.route_patterns.get(method));

        // Get the first match and return it
        Some((&routes, route_patterns.matches(uri).into_iter()))
    }
}

impl Handler for Router {
    type Result = BoxFuture<Response, hyper::Error>;

    #[inline]
    fn call(&self, ctx: Context) -> Self::Result {
        let filter_results = { self.filter(ctx.method(), ctx.path()) };
        if let Some((routes, route_indexes)) = filter_results {
            for route_index in route_indexes {
                let route = &routes[route_index];

                // TODO: The idea here is to eventually determine if there was a _parameter parse_
                //       error and continue iterating over matching routes

                // Re-parse the path to pull out captures
                if let Some(captures) = route.pattern().captures(ctx.path()) {
                } else {
                    // NOTE: This shouldn't be possible to fail as we already matched against the
                    //       path once. In the pathological case that we do fail here, stop
                    //       trying as the universe is probably positioned weirdly.
                    break;
                }

                return route.call(ctx);
            }
        }

        // Return 404 if we failed to find a matching route
        Box::new(future::ok(Response::with(StatusCode::NotFound)))
    }
}

#[cfg(test)]
mod tests {
    use super::Router;
    use Method::*;
    use Context;

    // Empty handler to use for route tests
    fn empty_handler(_: Context) {}

    /// Test for _some_ match for static in GET
    #[test]
    fn test_static_get() {
        let mut router = Router::new();
        router.add((Get, "/hello", empty_handler));

        assert!(router.find(&Get, "/hello").is_some());
        assert!(router.find(&Get, "/aa").is_none());
        assert!(router.find(&Get, "/hello/asfa").is_none());
    }

    /// Test for _some_ match for static in PUT, POST, DELETE
    #[test]
    fn test_static_put_post_del() {
        let mut router = Router::new();
        router.add((Put, "/hello", empty_handler));
        router.add((Post, "/hello", empty_handler));
        router.add((Delete, "/hello", empty_handler));

        assert!(router.find(&Get, "/hello").is_none());
        assert!(router.find(&Put, "/hello").is_some());
        assert!(router.find(&Post, "/hello").is_some());
        assert!(router.find(&Delete, "/hello").is_some());
    }

    /// Test for the correct match for static
    #[test]
    fn test_static_find() {
        // Correct match
        let mut router = Router::new();
        router.add((Get, "/aa", empty_handler));
        router.add((Get, "/hello", empty_handler));

        // FIXME: This section currently matches against regex
        //        This is an implementation detail; store the source strings and we'll
        //        match against that
        assert_eq!(router.find(&Get, "/hello").unwrap().pattern().as_str(), "^/hello$");
        assert_eq!(router.find(&Get, "/aa").unwrap().pattern().as_str(), "^/aa$");
    }

    /// Test for some match for segment parameter
    #[test]
    fn test_param_get() {
        let mut router = Router::new();
        router.add((Get, "/user/{id}", empty_handler));

        assert!(router.find(&Get, "/user/asfa").is_some());
        assert!(router.find(&Get, "/user/profile").is_some());
        assert!(router.find(&Get, "/user/3289").is_some());
        assert!(router.find(&Get, "/user").is_none());
    }
}
