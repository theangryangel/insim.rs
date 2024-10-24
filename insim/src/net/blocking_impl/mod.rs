use std::{
    fmt::Debug,
    io::{Read, Write},
};

pub(crate) mod framed;
pub(crate) mod udp;

/// Read Write super trait
pub trait ReadWrite: Read + Write + Debug {}
impl<T: Read + Write + Debug> ReadWrite for T {}

pub use framed::Framed;
pub use udp::UdpStream;
