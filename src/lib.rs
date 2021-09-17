mod client;
pub mod codec;
mod impl_string;
mod proto;

// Public API

pub use crate::client::Client;
pub use crate::proto::Insim as Packets;

// Traits

// TODO: Use our own errors
use std::io;
use std::string::FromUtf8Error;

pub trait InsimString {
    fn from_lfs(value: Vec<u8>) -> Result<String, FromUtf8Error>;
    fn to_lfs(&self, max_size: usize) -> Result<Vec<u8>, io::Error>;
}
