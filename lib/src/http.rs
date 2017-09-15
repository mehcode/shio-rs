//! Types that map directly to concepts in HTTP.
//!
//! This module exports types that map to HTTP concepts or to the underlying HTTP library
//! when needed.

// HTTP Methods
pub use hyper::Method;

// HTTP Headers
pub use hyper::header;

// HTTP Status Codes
pub use hyper::StatusCode;
