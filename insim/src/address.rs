//! Handle ToSocketAddrs ownership issues
use std::{
    io,
    net::{IpAddr, SocketAddr},
};

/// Deal with ToSocketAddrs
#[derive(Debug)]
pub enum Addr {
    /// SocketAddr
    SocketAddr(SocketAddr),
    /// IpAddr
    IpAddr(IpAddr, u16),
    /// Anything string-y
    String(String),
    /// A (host, port) tuple
    Tuple(String, u16),
}

impl ::std::net::ToSocketAddrs for Addr {
    type Iter = std::vec::IntoIter<SocketAddr>;

    fn to_socket_addrs(&self) -> io::Result<Self::Iter> {
        match self {
            Addr::SocketAddr(addr) => Ok(vec![*addr].into_iter()),
            Addr::IpAddr(host, port) => (*host, *port)
                .to_socket_addrs()
                .map(|iter| iter.collect::<Vec<_>>().into_iter()),
            Addr::String(s) => s
                .to_socket_addrs()
                .map(|iter| iter.collect::<Vec<_>>().into_iter()),
            Addr::Tuple(host, port) => (host.as_str(), *port)
                .to_socket_addrs()
                .map(|iter| iter.collect::<Vec<_>>().into_iter()),
        }
    }
}

impl From<(&str, u16)> for Addr {
    fn from(value: (&str, u16)) -> Self {
        Self::Tuple(value.0.to_owned(), value.1)
    }
}

impl From<(IpAddr, u16)> for Addr {
    fn from(value: (IpAddr, u16)) -> Self {
        Self::IpAddr(value.0, value.1)
    }
}

impl From<(String, u16)> for Addr {
    fn from(value: (String, u16)) -> Self {
        Self::Tuple(value.0, value.1)
    }
}

impl From<SocketAddr> for Addr {
    fn from(value: SocketAddr) -> Self {
        Self::SocketAddr(value)
    }
}

impl From<&str> for Addr {
    fn from(value: &str) -> Self {
        Self::String(value.to_owned())
    }
}

impl From<String> for Addr {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}
