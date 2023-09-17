use crate::result::Result;
use insim_core::{identifiers::RequestId, Decodable, Encodable};
use std::fmt::Debug;

pub trait Packets: Encodable + Decodable + Debug + Clone + Sized + Send + Sync {
    type Init: Encodable + Decodable + Debug + Clone + Sized + Send + Sync + Default + Into<Self>;

    fn is_ping(&self) -> bool;

    /// Create a pong
    fn pong(reqi: Option<RequestId>) -> Self;

    /// Maybe verify version
    fn maybe_verify_version(&self) -> Result<bool>;
}
