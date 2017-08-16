use std::collections::HashMap;

use hyper::{self, Method, StatusCode, Response};
use futures::{future, Future};

use route::Route;
use handler::Handler;
use context::Context;
use regex::RegexSet;

// From: https://github.com/crumblingstatue/try_opt/blob/master/src/lib.rs#L30
macro_rules! try_opt {
    ($e:expr) =>(
        match $e {
            Some(v) => v,
            None => return None,
        }
    )
}

#[derive(Clone, Default)]
pub struct Router {
    routes: HashMap<Method, Vec<Route>>,
    route_patterns: HashMap<Method, RegexSet>,
}

impl Router {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn add<R: Into<Route>>(&mut self, route: R) {
        let route: Route = route.into();
        let method = route.method().clone();

        self.routes
            .entry(method.clone())
            .or_insert(Vec::new())
            .push(route);

        let routes = &self.routes[&method];

        // TODO: It probably makes sense to make this router have some sort of compile step..
        // NOTE: The .unwrap cannot fail as we are using route patterns that are pre-verified
        self.route_patterns.insert(
            method.clone(),
            RegexSet::new(routes.iter().map(|r| r.pattern())).unwrap(),
        );
    }

    pub(crate) fn find(&self, method: &Method, path: &str) -> Option<&Route> {
        // Pull out the patterns and routes for this method
        let route_patterns = try_opt!(self.route_patterns.get(method));
        let routes = try_opt!(self.routes.get(method));

        // Get the first match and return it
        let matched_index = try_opt!(route_patterns.matches(path).into_iter().next());
        Some(&routes[matched_index])
    }
}

impl Handler for Router {
    type Future = Box<Future<Item = Response, Error = hyper::Error>>;

    #[inline]
    fn call(&self, ctx: Context) -> Self::Future {
        let route = match self.find(ctx.method(), ctx.path()) {
            Some(route) => route,
            None => {
                // Return 404 if we failed to find a matching route
                return Box::new(future::ok(Response::new().with_status(StatusCode::NotFound)));
            }
        };

        route.call(ctx)
    }
}
