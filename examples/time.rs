extern crate shio;

use std::time;

use shio::Stack;
use shio::prelude::*;

fn hello(_: Context) -> Response {
    Response::with("Hello World!\n")
}

// Measures request time in μs and prints it out
fn timeit(next: BoxHandler) -> BoxHandler {
    Box::new(move |ctx: Context| -> BoxFutureResponse {
        let time_before = time::Instant::now();

        // TODO: Use `.inspect` over `.map` when
        //  https://github.com/alexcrichton/futures-rs/pull/565 is merged and available in
        //  a released version

        Box::new(next.call(ctx).map(move |response| {
            let d = time::Instant::now().duration_since(time_before);
            let elapsed = (d.as_secs() * 1_000_000) + (d.subsec_nanos() as u64 / 1_000);
            println!("Request took {}μs", elapsed);

            response
        }))
    })
}

fn main() {
    // Create a new middleware stack around `hello` ..
    let stack = Stack::new(hello)
        // .. with `timeit`
        .with(timeit);

    // Create and run a Shio service, using our configured stack as _its_ handler
    Shio::new(stack).run(":7878").unwrap();
}
