use std::sync::Arc;
use std::borrow::Cow;

use futures::{future, Future};
use http::method::Method;
use hyper::{self, Response, StatusCode};
use regex::Regex;

use super::{Context, Handler};

#[derive(Clone)]
pub struct Route {
    method: Method,
    pattern: Regex,
    handler: Arc<Handler<Future = Box<Future<Item = Response, Error = hyper::Error>>>>,
}

impl Route {
    pub fn new<P, H>(method: Method, pattern: P, handler: H) -> Self
    where
        P: AsRef<str>,
        H: Handler + 'static,
    {
        Route {
            handler: Arc::new(RouteHandler(handler)),
            // TODO: Does this unwrap make sense?
            pattern: Regex::new(&normalize_pattern(pattern.as_ref())).unwrap(),
            method,
        }
    }

    #[inline]
    pub fn method(&self) -> &Method {
        &self.method
    }

    #[inline]
    pub fn pattern(&self) -> &str {
        self.pattern.as_str()
    }
}

impl<P, H> From<(Method, P, H)> for Route
where
    P: AsRef<str>,
    H: Handler + 'static,
{
    fn from(arguments: (Method, P, H)) -> Self {
        Route::new(arguments.0, arguments.1, arguments.2)
    }
}

impl Handler for Route {
    type Future = Box<Future<Item = Response, Error = hyper::Error>>;

    #[inline]
    fn call(&self, ctx: Context) -> Self::Future {
        self.handler.call(ctx)
    }
}

struct RouteHandler<H: Handler>(H);

impl<H: Handler> Handler for RouteHandler<H>
where
    <H as Handler>::Future: 'static,
{
    type Future = Box<Future<Item = Response, Error = hyper::Error>>;

    #[inline]
    fn call(&self, ctx: Context) -> Self::Future {
        Box::new(self.0.call(ctx).or_else(|_| {
            // FIXME: Do something with the error argument. Perhaps require at least `:Debug`
            //        so we can let someone know they hit the default error catcher

            future::ok(Response::new().with_status(StatusCode::InternalServerError))
        }))
    }
}

// Copied initial impl from https://github.com/ubnt-intrepid/susanoo/blob/master/lib/src/router/route.rs#L111
// TODO: Rework patterns quite a bit so they can support simplified matches
//       e.g. "/<id>" or "/<filename: .*>"
fn normalize_pattern(pattern: &str) -> Cow<str> {
    if pattern == "" {
        // A pattern of "" means <anything goes> and can be used as final fallback route
        "".into() }
    else {
        let pattern = pattern.trim().trim_left_matches("^").trim_right_matches("$").trim_right_matches("/");

        match pattern {
            "" => "^/$".into(),
            s => format!("^{}/?$", s).into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::normalize_pattern;

    #[test]
    fn normalize_pattern() {
        assert_eq!(normalize_pattern(""), "");
        assert_eq!(normalize_pattern("/"), "^/$");
        assert_eq!(normalize_pattern("/path/to"), "^/path/to/?$");
        assert_eq!(normalize_pattern("/path/to/"), "^/path/to/?$");
    }
}
