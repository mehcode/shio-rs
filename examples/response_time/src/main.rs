extern crate shio;

use std::time::Instant;

use shio::Stack;
use shio::prelude::*;

fn hello(_: &Context) -> Response {
    Response::with("Hello World!\n")
}

// Measures request time in μs and prints it out
fn timeit(next: BoxHandlerMut) -> BoxHandlerMut {
    (move |ctx: &mut Context| {
        let time_before = Instant::now();

        next.call(ctx)
            .inspect(move |_| {
                let d = Instant::now().duration_since(time_before);
                let elapsed = (d.as_secs() * 1_000_000) + (d.subsec_nanos() as u64 / 1_000);
                println!("Request took {}μs", elapsed);
            })
            .into_box()
    }).into_box()
}

fn main() {
    // Create a new middleware stack around `hello` ..
    let stack = Stack::new(hello)
        // .. with `timeit`
        .with(timeit);

    // Create and run a Shio service, using our configured stack as _its_ handler
    Shio::new(stack).run(":7878").unwrap();
}
