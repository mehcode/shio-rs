use std::fmt;

use hyper;
use futures::{future, Future, IntoFuture};

use response::Response;
use StatusCode;
use header::ContentLength;
use ext::{BoxFuture, FutureExt, IntoFutureExt};
use handler::default_catch;

pub trait Responder<'r>
where
    <Self::Result as IntoFuture>::Error: fmt::Debug + Send + Sync,
{
    type Result: IntoFuture<Item = Response> + 'r;

    fn to_response(self) -> Self::Result;
}

impl<'r> Responder<'r> for () {
    type Result = Response;

    #[inline]
    fn to_response(self) -> Self::Result {
        Response::build().status(StatusCode::NoContent).into()
    }
}

impl<'r, 'a> Responder<'r> for &'a str {
    type Result = Response;

    #[inline]
    fn to_response(self) -> Self::Result {
        self.to_owned().to_response()
    }
}

impl<'r> Responder<'r> for String {
    type Result = Response;

    #[inline]
    fn to_response(self) -> Self::Result {
        Response::build()
            .header(ContentLength(self.len() as u64))
            .body(self)
    }
}

impl<'r> Responder<'r> for StatusCode {
    type Result = Response;

    #[inline]
    fn to_response(self) -> Self::Result {
        Response::build().status(self).into()
    }
}

impl<'r, R: Responder<'r>> Responder<'r> for (StatusCode, R)
where
    <R::Result as IntoFuture>::Error: fmt::Debug + Send + Sync,
{
    type Result = BoxFuture<'r, <R::Result as IntoFuture>::Item, <R::Result as IntoFuture>::Error>;

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

impl<'r> Responder<'r> for Response {
    type Result = Response;

    #[inline]
    fn to_response(self) -> Self::Result {
        self
    }
}

impl<'r, E, R> Responder<'r> for Box<Future<Item = R, Error = E> + 'r>
where
    E: fmt::Debug + Send + Sync + 'r,
    R: Responder<'r> + 'r,
    <R::Result as IntoFuture>::Error: fmt::Debug + Send + Sync,
{
    type Result = Box<Future<Item = Response, Error = hyper::Error> + 'r>;

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

impl<'r, R: Responder<'r>> IntoFutureExt<'r, Response> for R
where
    <R::Result as IntoFuture>::Error: fmt::Debug + Send + Sync,
{
    type Error = <R::Result as IntoFuture>::Error;
    type Future = <R::Result as IntoFuture>::Future;

    fn into_future_ext(self) -> Self::Future {
        self.to_response().into_future()
    }
}

#[cfg(not(feature = "nightly"))]
impl<'r, E, R> Responder<'r> for Result<R, E>
where
    E: fmt::Debug + Send + Sync,
    R: Responder<'r>,
    <R::Result as IntoFuture>::Error: fmt::Debug + Send + Sync,
{
    type Result = BoxFuture<'r, <R::Result as IntoFuture>::Item, hyper::Error>;

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

// #[cfg(feature = "nightly")]
// impl<E, R> Responder for Result<R, E>
// where
//     E: fmt::Debug + Send + Sync,
//     R: Responder,
//     <<R as Responder>::Result as IntoFuture>::Error: fmt::Debug + Send + Sync,
// {
//     type Error = hyper::Error;
//     type Result = BoxFuture<<R::Result as IntoFuture>::Item, E>;
//
//     #[inline]
//     default fn to_response(self) -> Self::Result {
//         match self {
//             Ok(responder) => responder
//                 .to_response()
//                 .into_future()
//                 .or_else(default_catch)
//                 .into_box(),
//             Err(error) => future::ok(default_catch(error)).into_box(),
//         }
//     }
// }
//
// #[cfg(feature = "nightly")]
// impl<E, R> Responder for Result<R, E>
// where
//     E: Responder + fmt::Debug + Send + Sync,
//     R: Responder,
//     <<R as Responder>::Result as IntoFuture>::Error: fmt::Debug + Send + Sync,
// {
//     #[inline]
//     fn to_response(self) -> Self::Result {
//         match self {
//             Ok(responder) => responder
//                 .to_response()
//                 .into_future()
//                 .or_else(default_catch)
//                 .into_box(),
//
//             Err(eresponder) => eresponder
//                 .to_response()
//                 .into_future()
//                 .or_else(default_catch)
//                 .into_box(),
//         }
//     }
// }

#[cfg(test)]
mod tests {
    use std::fmt;

    use tokio_core::reactor::Core;
    use futures::{Future, IntoFuture, Stream};

    use super::{Responder, Response, StatusCode};

    fn to_response<R: Responder>(r: R) -> Response
    where
        <R::Result as IntoFuture>::Error: fmt::Debug + Send + Sync,
    {
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
