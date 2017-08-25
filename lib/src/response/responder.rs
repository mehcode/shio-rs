use std::fmt;

use hyper;
use futures::{future, Future, IntoFuture};

use response::Response;
use StatusCode;
use header::ContentLength;
use ext::{BoxFuture, FutureExt, IntoFutureExt};
use handler::default_catch;

pub trait Responder {
    type Error: fmt::Debug + Send + Sync;
    type Result: IntoFuture<Item = Response, Error = Self::Error>;

    fn to_response(self) -> Self::Result;
}

impl<'a> Responder for &'a str {
    type Error = hyper::Error;
    type Result = Response;

    #[inline]
    fn to_response(self) -> Self::Result {
        self.to_owned().to_response()
    }
}

impl Responder for String {
    type Error = hyper::Error;
    type Result = Response;

    #[inline]
    fn to_response(self) -> Self::Result {
        Response::build()
            .header(ContentLength(self.len() as u64))
            .body(self)
    }
}

impl Responder for StatusCode {
    type Error = hyper::Error;
    type Result = Response;

    #[inline]
    fn to_response(self) -> Self::Result {
        Response::build().status(self).into()
    }
}

impl<E: fmt::Debug + Send + Sync, R: Responder<Error = E>> Responder for (StatusCode, R)
where
    E: 'static,
    <<R as Responder>::Result as IntoFuture>::Future: 'static,
{
    type Error = E;
    type Result = BoxFuture<<R::Result as IntoFuture>::Item, Self::Error>;

    #[inline]
    fn to_response(self) -> Self::Result {
        let (status, responder) = self;
        responder
            .to_response()
            .into_future()
            .map(move |mut response| {
                response.set_status(status);
                response
            })
            .into_box()
    }
}

impl Responder for Response {
    type Error = hyper::Error;
    type Result = Response;

    #[inline]
    fn to_response(self) -> Self::Result {
        self
    }
}

impl<E: fmt::Debug + Send + Sync, R: Responder> Responder for Box<Future<Item = R, Error = E>>
where
    E: 'static,
    R: 'static,
    <<R as Responder>::Result as IntoFuture>::Error: 'static,
    <<R as Responder>::Result as IntoFuture>::Future: 'static,
{
    type Error = hyper::Error;
    type Result = Box<Future<Item = Response, Error = Self::Error>>;

    #[inline]
    fn to_response(self) -> Self::Result {
        self.then(|result| match result {
            Ok(responder) => responder
                .to_response()
                .into_future()
                .or_else(default_catch)
                .into_box(),
            Err(error) => future::ok(default_catch(error)).into_box(),
        }).into_box()
    }
}

impl<R: Responder + 'static> IntoFutureExt<Response> for R {
    type Error = R::Error;
    type Future = <R::Result as IntoFuture>::Future;

    fn into_future_ext(self) -> Self::Future {
        self.to_response().into_future()
    }
}

impl<E: fmt::Debug + Send + Sync, R: Responder> Responder for Result<R, E>
where
    E: 'static,
    <<R as Responder>::Result as IntoFuture>::Error: 'static,
    <<R as Responder>::Result as IntoFuture>::Future: 'static,
{
    type Error = hyper::Error;
    type Result = BoxFuture<<R::Result as IntoFuture>::Item, Self::Error>;

    #[inline]
    fn to_response(self) -> Self::Result {
        match self {
            Ok(responder) => responder
                .to_response()
                .into_future()
                .or_else(default_catch)
                .into_box(),
            Err(error) => future::ok(default_catch(error)).into_box(),
        }
    }
}

#[cfg(test)]
mod tests {
    use tokio_core::reactor::Core;
    use futures::{Future, IntoFuture, Stream};

    use super::{Responder, Response, StatusCode};

    fn to_response<R: Responder>(r: R) -> Response {
        Core::new()
            .unwrap()
            .run(r.to_response().into_future())
            .unwrap()
    }

    fn assert_body(res: Response, expected: &str) {
        let mut core = Core::new().unwrap();

        let work = res.body().concat2().map(|body| {
            let body = String::from_utf8_lossy(&body);

            assert_eq!(expected, body);
        });

        core.run(work).unwrap();
    }

    #[test]
    fn str_to_response() {
        let res = to_response("Hello\n");

        assert_eq!(res.status(), StatusCode::Ok);
        assert_body(res, "Hello\n");
    }

    #[test]
    fn string_to_response() {
        let res = to_response(String::from("Hello\n"));

        assert_eq!(res.status(), StatusCode::Ok);
        assert_body(res, "Hello\n");
    }

    #[test]
    fn pair_to_response() {
        let res = to_response((StatusCode::Accepted, "Hello\n"));

        assert_eq!(res.status(), StatusCode::Accepted);
        assert_body(res, "Hello\n");
    }

    #[test]
    fn status_to_response() {
        let res = to_response(StatusCode::NoContent);

        assert_eq!(res.status(), StatusCode::NoContent);
        assert_body(res, "");
    }
}
