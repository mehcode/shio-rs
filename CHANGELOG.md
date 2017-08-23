# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/)
and this project adheres to [Semantic Versioning](http://semver.org/spec/v2.0.0.html).

## [0.0.5] - 2017-08-23
### Added
  - Added `response::Builder`. Construct with `Response::build()`.

    ```rust
    // This ...
    Response::new().with_status(StatusCode::Ok).with_body("Hello World\n");

    // Becomes ...
    Response::build().status(StatusCode::Ok).body("Hello World\n");
    ```

  - Recover panicked worker threads and respawn
  - Log handler error when received by the default error catcher
  - Add `BoxFuture<T, E>` and `FutureExt::into_box` to try and ease box construction to write handlers when not using `impl Trait`. Any `Future` may have `.into_box` applied to it to turn it into a `Box<Future>`.

    ```rust
    fn proxy_google(ctx: Context) -> BoxFutureResponse<hyper::Error> {
        Client::new(&ctx)
            .get("http://www.google.com".parse().unwrap())
            .map(|res| Response::build().body(res.body()))
            // Future turned into Box<Future<Item = Response, Error = hyper::Error>>
            // which BoxFutureResponse<hyper::Error> is an alias of
            .into_box()
    }
    ```

### Changed
  - Require `Debug + Send + Sync` on errors returned from handlers
  - Renamed `StackHandler` to `Middleware`
  - Renamed `Stack::add` to `Stack::with` and optimized usage for builder pattern

    ```rust
    let mut stack = Stack::new(handler).with(middleware).with(other_middleware);
    stack = stack.with(yet_more_middleware);
    ```

  - Renamed `Handler::boxed` to `Handler::into_box` to follow API guidelines

### Removed
  - Removed `Response::with_*` methods.

## [0.0.4] - 2017-08-22
### Changed
  - Renamed to `Shio` from [/u/xav_19](https://www.reddit.com/u/xav_19)

### Fixed
  - [Correction](https://github.com/mehcode/shio-rs/pull/2) on [hello example](https://github.com/mehcode/shio-rs/blob/v0.0.4/examples/hello.rs) from [@frewsxcv](https://github.com/frewsxcv)

## [0.0.3] - 2017-08-21
### Added
  - Add `hyper` to process the HTTP protocol.
  - Add basic `Router`. Does not currently handle URL parameters.
  - Designate the `Default` handler for our service to be an instance of `Router`.
  - Add `Stack` as a middleware container.
  - Add `ToSocketAddrsExt` to allow using `:<port>` as a valid address and defaulting the ip to both `0.0.0.0` and `::0`.

### Changed
  - HTTP request properties added to `Context`.
  - `Handler` is now required to return a `Response`, either directly or with a Future.

## [0.0.2] - 2017-08-13
### Changed
  - Expanded `Handler` to accept a `Context` which is the request/connection plus the a handle to the thread local event loop.

## 0.0.1 - 2017-08-13
### Added
  - Asynchronous `Handler` that can be a simple function.
  - Service for `tokio` that is a multithreaded abstraction over `Handler`.

[Unreleased]: ../../compare/v0.0.5...HEAD
[0.0.5]: ../../compare/v0.0.4...v0.0.5
[0.0.4]: ../../compare/v0.0.3...v0.0.4
[0.0.3]: ../../compare/v0.0.2...v0.0.3
[0.0.2]: ../../compare/v0.0.1...v0.0.2
