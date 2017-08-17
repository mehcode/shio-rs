use std::borrow::Cow;

use futures::Future;
use hyper::{self, Method};
use regex::Regex;

use super::{Context, Handler, BoxHandler};

//#[derive(Clone)]
pub struct Route {
    method: Method,
    pattern: Regex,
    handler: BoxHandler,
}

impl Route {
    pub fn new<P, H>(method: Method, pattern: P, handler: H) -> Self
    where
        P: AsRef<str>,
        H: Handler + 'static,
    {
        Route {
            handler: handler.boxed(),
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
    type Future = Box<Future<Item = hyper::Response, Error = hyper::Error>>;

    #[inline]
    fn call(&self, ctx: Context) -> Self::Future {
        self.handler.call(ctx)
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
