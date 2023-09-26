use crate::result::Result;
use insim_core::{Decodable, Encodable};
use std::fmt::Debug;

pub trait Frame: Encodable + Decodable + Debug + Clone + Sized + Send + Sync {
    type Init: Debug + Clone + Sized + Send + Sync + Default + Into<Self>;

    fn maybe_pong(&self) -> Option<Self>;

    /// Maybe verify version
    fn maybe_verify_version(&self) -> Result<bool>;
}
