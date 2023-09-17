use crate::result::Result;
use insim_core::{identifiers::RequestId, Decodable, Encodable};
use std::fmt::Debug;

pub trait Packets: Encodable + Decodable + Debug + Clone + Sized + Send + Sync {
    fn is_ping(&self) -> bool;

    /// Create a pong
    fn pong(reqi: Option<RequestId>) -> Self;

    /// Maybe verify version
    fn maybe_verify_version(&self) -> Result<bool>;
}

pub trait Init: Encodable + Decodable + Debug + Clone + Sized + Send + Sync {}
