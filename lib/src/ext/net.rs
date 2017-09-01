use std::net::{SocketAddr, ToSocketAddrs};
use std::io;

/// An extension of [`ToSocketAddrs`] that allows for a default address when specifying just
/// the port as `:8080`.
///
/// [`ToSocketAddrs`]: https://doc.rust-lang.org/std/net/trait.ToSocketAddrs.html
pub trait ToSocketAddrsExt {
    type Iter: Iterator<Item = SocketAddr>;

    fn to_socket_addrs_ext(&self) -> io::Result<Self::Iter>;
}

impl<'a> ToSocketAddrsExt for &'a str {
    type Iter = <Self as ToSocketAddrs>::Iter;

    #[cfg_attr(feature = "cargo-clippy", allow(result_unwrap_used))]
    fn to_socket_addrs_ext(&self) -> io::Result<Self::Iter> {
        if self.starts_with(':') {
            // If we start with `:`; assume the ip is omitted and this is just a port
            // specification
            let port: u16 = self[1..]
                .parse()
                .map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;

            Ok(
                (&[
                    SocketAddr::new("0.0.0.0".parse().unwrap(), port),
                    SocketAddr::new("::0".parse().unwrap(), port),
                ][..])
                    .to_socket_addrs()?
                    .collect::<Vec<_>>()
                    .into_iter(),
            )
        } else {
            self.to_socket_addrs()
        }
    }
}

impl ToSocketAddrsExt for String {
    type Iter = <Self as ToSocketAddrs>::Iter;

    fn to_socket_addrs_ext(&self) -> io::Result<Self::Iter> {
        (&**self).to_socket_addrs_ext()
    }
}

macro_rules! forward_to_socket_addrs {
    ($lifetime:tt, $ty:ty) => (
        impl<$lifetime> ToSocketAddrsExt for $ty {
            type Iter = <$ty as ToSocketAddrs>::Iter;

            fn to_socket_addrs_ext(&self) -> io::Result<Self::Iter> {
                self.to_socket_addrs()
            }
        }
    );
}

forward_to_socket_addrs!('a, &'a [SocketAddr]);
forward_to_socket_addrs!('a, (&'a str, u16));

#[cfg(test)]
mod tests {
    use super::ToSocketAddrsExt;

    #[test]
    fn to_socket_addrs_ext_str() {
        let addresses = ":7878".to_socket_addrs_ext().unwrap().collect::<Vec<_>>();

        assert_eq!(addresses.len(), 2);
        assert_eq!(addresses[0], "0.0.0.0:7878".parse().unwrap());
        assert_eq!(addresses[1], "[::0]:7878".parse().unwrap());
    }

    #[test]
    fn to_socket_addrs_ext_string() {
        let address = ":7878".to_owned();
        let addresses = address.to_socket_addrs_ext().unwrap().collect::<Vec<_>>();

        assert_eq!(addresses.len(), 2);
        assert_eq!(addresses[0], "0.0.0.0:7878".parse().unwrap());
        assert_eq!(addresses[1], "[::0]:7878".parse().unwrap());
    }
}
