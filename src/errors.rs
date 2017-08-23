//! Types representing errors that can occur from inside Shio.

use std::fmt;
use std::error::Error;
use std::io;

/// An error that occurs during `Shio::listen` or `Shio::run`.
#[derive(Debug)]
pub struct ListenError {
    inner: ListenErrorKind,
}

impl fmt::Display for ListenError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.inner {
            ListenErrorKind::Io(ref err) => err.fmt(f),
        }
    }
}

impl Error for ListenError {
    fn description(&self) -> &str {
        match self.inner {
            ListenErrorKind::Io(ref err) => err.description(),
        }
    }

    fn cause(&self) -> Option<&Error> {
        match self.inner {
            ListenErrorKind::Io(ref err) => err.cause(),
        }
    }
}

impl<T: Into<ListenErrorKind>> From<T> for ListenError {
    fn from(err: T) -> Self {
        ListenError { inner: err.into() }
    }
}

#[derive(Debug)]
pub enum ListenErrorKind {
    Io(io::Error),
}

impl From<io::Error> for ListenErrorKind {
    fn from(err: io::Error) -> Self {
        ListenErrorKind::Io(err)
    }
}
