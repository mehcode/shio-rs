# Shio
![Rust](https://img.shields.io/badge/rust-stable-brightgreen.svg)
[![Crates.io](https://img.shields.io/crates/v/shio.svg)](https://crates.io/crates/shio)
[![Crates.io](https://img.shields.io/crates/d/shio.svg)](https://crates.io/crates/shio)
[![Docs.rs](https://docs.rs/shio/badge.svg)](https://docs.rs/shio)
[![IRC](https://img.shields.io/badge/chat-%23shio-yellow.svg)](https://kiwiirc.com/client/irc.mozilla.org/#shio)
> Shio is a fast, simple, and asynchronous micro web-framework for Rust.

 - **Asynchronous**. Handlers are both handled _asynchronously_ and may be _asynchronous_ themselves. A `shio::Handler` receives a `tokio_core::reactor::Handle` which may be used to schedule additional work on the thread-local event loop.

 - **Multithreaded**. By default, requests are handled by multiple threads, each running an event loop powered by `tokio`.

### WARNING: Shio is at 0.0.x which means the API is highly unstable. Use at your own risk. See [#1](https://github.com/mehcode/shio-rs/issues/1) to discuss our general direction.

## Usage

```toml
[dependencies]
shio = "0.0.6"
```

## Example

```rust
extern crate shio;

use shio::prelude::*;

fn hello_world(_: Context) -> &'static str {
  "Hello World\n"
}

fn main() {
  Shio::default().route((Method::Get, "/", hello_world)).run(":7878").unwrap();
}
```

See the [examples](https://github.com/mehcode/shio-rs/tree/master/examples) for more usage information.

To run a specific example, use:

```bash
$ cargo run -p <example name>
```

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
