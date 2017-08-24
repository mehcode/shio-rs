use std::fmt;

use hyper;
use futures::{future, Future, IntoFuture};

use response::Response;
use StatusCode;
use header::ContentLength;
use ext::{BoxFuture, FutureExt};

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

impl<E: fmt::Debug + Send + Sync, R: Responder<Error = E>> Responder for Result<R, E>
where
    E: 'static,
    <<R as Responder>::Result as IntoFuture>::Future: 'static,
{
    type Error = E;
    type Result = BoxFuture<<R::Result as IntoFuture>::Item, Self::Error>;

    #[inline]
    fn to_response(self) -> Self::Result {
        match self {
            Ok(responder) => responder.to_response().into_future().into_box(),
            Err(error) => future::err(error).into_box(),
        }
    }
}

#[cfg(test)]
mod tests {
    use tokio_core::reactor::Core;
    use futures::{Future, Stream};

    use super::{Responder, Response, StatusCode};

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
        let res = "Hello\n".to_response();

        assert_eq!(res.status(), StatusCode::Ok);
        assert_body(res, "Hello\n");
    }

    #[test]
    fn string_to_response() {
        let res = String::from("Hello\n").to_response();

        assert_eq!(res.status(), StatusCode::Ok);
        assert_body(res, "Hello\n");
    }

    #[test]
    fn pair_to_response() {
        let res = (StatusCode::Accepted, "Hello\n").to_response();

        assert_eq!(res.status(), StatusCode::Accepted);
        assert_body(res, "Hello\n");
    }

    #[test]
    fn status_to_response() {
        let res = StatusCode::NoContent.to_response();

        assert_eq!(res.status(), StatusCode::NoContent);
        assert_body(res, "");
    }
}
