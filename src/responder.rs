use hyper::header::ContentLength;
use hyper::StatusCode;

use response::Response;

pub trait Responder {
    fn to_response(self) -> Response;
}

impl<'a> Responder for &'a str {
    #[inline]
    fn to_response(self) -> Response {
        self.to_owned().to_response()
    }
}

impl Responder for String {
    #[inline]
    fn to_response(self) -> Response {
        Response::new()
            .with_header(ContentLength(self.len() as u64))
            .with_body(self)
    }
}

impl Responder for StatusCode {
    #[inline]
    fn to_response(self) -> Response {
        Response::new().with_status(self)
    }
}

impl<R: Responder> Responder for (StatusCode, R) {
    #[inline]
    fn to_response(self) -> Response {
        self.1.to_response().with_status(self.0)
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
