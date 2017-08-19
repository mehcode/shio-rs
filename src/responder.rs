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
            .header(ContentLength(self.len() as u64))
            .body(self)
    }
}

impl Responder for StatusCode {
    #[inline]
    fn to_response(self) -> Response {
        Response::new()
            .status(self)
    }
}
