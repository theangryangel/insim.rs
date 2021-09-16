pub mod codec;
mod client;
mod proto;
mod impl_string;

// Public API

pub use crate::client::Client;
pub use crate::proto::Insim as Packets;

// Traits

// TODO: Use our own errors
use std::string::FromUtf8Error;
use std::io;

pub trait InsimString {
    fn from_lfs(value: Vec<u8>) -> Result<String, FromUtf8Error>;
    fn to_lfs(&self, max_size: usize) -> Result<Vec<u8>, io::Error>;
}
