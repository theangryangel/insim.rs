use std::fmt::Debug;
use insim_core::{Encodable, Decodable, identifiers::RequestId};
use crate::result::Result;

pub trait Packets: Encodable + Decodable + Debug + Clone + Sized {
    fn is_ping(&self) -> bool;

    /// Create a pong
    fn pong(reqi: Option<RequestId>) -> Self;

    /// Maybe verify version
    fn maybe_verify_version(&self) -> Result<bool>;
}

pub trait Init: Encodable + Decodable + Debug + Clone + Sized {}
