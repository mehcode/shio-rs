extern crate shio;

use shio::{Context, Handler, Method, Response, Shio};
use shio::ext::IntoFutureExt;
use shio::router::Router;

trait Middleware<H: Handler>: Send + Sync {
    type Result: IntoFutureExt<Response>;

    fn call(&self, context: Context, next: &H) -> Self::Result;
}

impl<H: Handler> Middleware<H> for () {
    type Result = H::Result;

    fn call(&self, context: Context, next: &H) -> Self::Result {
        println!("()!");

        next.call(context)
    }
}

#[derive(Debug)]
struct Stack<H, M = ()>(H, M)
where
    H: Handler,
    M: Middleware<H>;

impl<H: Handler, M: Middleware<H>> Handler for Stack<H, M> {
    type Result = M::Result;

    fn call(&self, context: Context) -> Self::Result {
        self.1.call(context, &self.0)
    }
}

#[derive(Debug)]
struct Number(u32);

impl shio::context::Key for Number {
    type Value = u32;
}

impl<H: Handler> Middleware<H> for Number {
    type Result = H::Result;

    fn call(&self, mut context: Context, next: &H) -> Self::Result {
        // Insert a number into the request context
        context.put::<Number>(self.0);

        next.call(context)
    }
}

#[derive(Debug)]
struct Multiply(u32);

impl<H: Handler> Middleware<H> for Multiply {
    type Result = H::Result;

    fn call(&self, mut context: Context, next: &H) -> Self::Result {
        println!("MULTIPLY: {}!", self.0);

        let value = *context.get::<Number>();
        context.put::<Number>(value * self.0);

        next.call(context)
    }
}

fn index(context: Context) {
    println!("recv: {:?}", context.try_get::<Number>());
}

fn main() {
    let mut router = Router::new();
    router.add((Method::Get, "/", index));

    // TODO: Dream up a nice way to construct this contraption

    let stack = Stack(
        Stack(Stack(Stack(router, ()), Multiply(3)), Multiply(2)),
        Number(231),
    );

    Shio::new(stack).run(":7878").unwrap();
}
