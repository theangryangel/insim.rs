use std::{
    fmt::Debug,
    io::{Read, Write},
};

pub(crate) mod framed;
pub(crate) mod udp;
pub(crate) mod websocket;

/// Read Write super trait
pub trait ReadWrite: Read + Write + Debug {}
impl<T: Read + Write + Debug> ReadWrite for T {}

pub use framed::Framed;
pub use udp::UdpStream;
pub use websocket::WebsocketStream;
