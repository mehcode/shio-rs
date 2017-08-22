# Salt
![Rust](https://img.shields.io/badge/rust-stable-brightgreen.svg)
[![Crates.io](https://img.shields.io/crates/v/salt.svg)](https://crates.io/crates/salt)
[![Crates.io](https://img.shields.io/crates/d/salt.svg)](https://crates.io/crates/salt)
[![Docs.rs](https://docs.rs/salt/badge.svg)](https://docs.rs/salt)
[![IRC](https://img.shields.io/badge/chat-%23salt-yellow.svg)](https://kiwiirc.com/client/irc.mozilla.org/#salt)
> Salt is a fast, simple, and asynchronous micro web-framework for Rust.

 - **Asynchronous**. Handlers are both handled _asynchronously_ and may be _asynchronous_ themselves. A `salt::Handler` receives a `tokio_core::reactor::Handle` which may be used to schedule additional work on the thread-local event loop.

 - **Multi-threaded**. By default, requests are handled by multiple threads, each running an event loop powered by `tokio`.

### WARNING: Salt is at 0.0.x which means the API is highly unstable. Use at your own risk. See [#1](https://github.com/mehcode/salt-rs/issues/1) to discuss our general direction.

## Usage

```toml
[dependencies]
salt = "0.0.3"
```

## Example

```rust
use salt::prelude::*;

fn hello_world(_: Context) -> Response {
  Response::with("Hello World\n")
}

fn main() {
  Salt::default().route((Method::Get, "/", hello_world)).run(":7878").unwrap();
}
```

See the [./examples](https://github.com/mehcode/salt-rs/tree/master/examples) for more usage information.

## License

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
