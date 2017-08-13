//! Example of how a remote CLI session could be implemented.

extern crate salt;
extern crate tokio_io;
extern crate tokio_core;
extern crate futures;

use std::io::BufReader;

use futures::{Future, Stream};
use tokio_io::{io, AsyncRead};

use salt::{Context, Salt};

fn main() {
    // By default, Salt will multiplex connections over <num_cpus> threads.
    // TODO: Investigate this behavior. It might be best to be opt-in.

    Salt::new(|c: Context| {
        // Split the stream into its reader and writer halves
        // This is needed because `BufReader` would otherwise consume the
        // stream and we'd lose write access
        let (reader, writer) = c.split();

        // Write an initial sigil, to show we can accept input
        // Notice that the io:: combinators consume the writer and pass it back
        io::write_all(writer, " > ").and_then(|(writer, _)| {
            // Map the reader into a stream that produces lines of content
            // We use `.fold` as a trick to keep shoving the writer
            // along with each pass of our line iterator
            let reader = BufReader::new(reader);
            io::lines(reader).fold(writer, |writer, line| {
                // Format the line to add a sigil before
                let output = format!(" < {}\n", line);

                // Write out each line that we received
                io::write_all(writer, output).and_then(|(writer, _)| {
                    // Write out the next leading sigil
                    io::write_all(writer, " > ")
                }).map(|(writer, _)| writer)
                // We need the final .map as our fold trick requires us to return
                // what we passed in to keep the cycle going
            })
        })
    })
        // Bind to `localhost:7878`. Depending on your hosts file,
        // this can resolve to `127.0.0.1:7878` _and_ `[::1]:7878`.
        //
        // Uses https://doc.rust-lang.org/stable/std/net/trait.ToSocketAddrs.html
        .listen("localhost:7878")
        .unwrap();

    // Note that `Salt#listen` is non-blocking.
    // You are free to spin up 20 Salt servers.
}
