use std::ops::Deref;
use tokio_core::net::TcpStream;
use tokio_core::reactor::Handle;
use tokio_io::{AsyncRead, AsyncWrite};
use std::io::{self, Read, Write};
use bytes::{Buf, BufMut};
use futures::Poll;

pub struct Context {
    handle: Handle,
    stream: TcpStream,
}

impl Context {
    pub(crate) fn new(handle: Handle, stream: TcpStream) -> Self {
        Context { handle, stream }
    }
}

impl Deref for Context {
    type Target = Handle;

    fn deref(&self) -> &Self::Target {
        &self.handle
    }
}

impl Read for Context {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.stream.read(buf)
    }
}

impl Write for Context {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.stream.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.stream.flush()
    }
}

impl AsyncRead for Context {
    unsafe fn prepare_uninitialized_buffer(&self, buffer: &mut [u8]) -> bool {
        self.stream.prepare_uninitialized_buffer(buffer)
    }

    fn read_buf<B: BufMut>(&mut self, buf: &mut B) -> Poll<usize, io::Error> {
        <&TcpStream>::read_buf(&mut &self.stream, buf)
    }
}

impl AsyncWrite for Context {
    fn shutdown(&mut self) -> Poll<(), io::Error> {
        <&TcpStream>::shutdown(&mut &self.stream)
    }

    fn write_buf<B: Buf>(&mut self, buf: &mut B) -> Poll<usize, io::Error> {
        <&TcpStream>::write_buf(&mut &self.stream, buf)
    }
}
