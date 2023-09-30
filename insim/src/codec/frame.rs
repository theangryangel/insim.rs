use crate::{result::Result, relay::HostSelect};
use insim_core::{Decodable, Encodable};
use std::fmt::Debug;

pub trait FrameInitData {}

pub trait Frame: Encodable + Decodable + Debug + Clone + Sized + Send + Sync + From<HostSelect> {
    type Isi: Debug + Clone + Sized + Send + Sync + Default + Into<Self> + FrameInitData;

    /// Maybe send a ping/pong response
    fn maybe_pong(&self) -> Option<Self>;

    /// Maybe verify version
    fn maybe_verify_version(&self) -> Result<bool>;

    fn isi_default() -> Self::Isi {
        Self::Isi::default()
    }
}
