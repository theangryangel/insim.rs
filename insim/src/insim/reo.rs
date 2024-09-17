use insim_core::binrw::{self, binrw};

use crate::identifiers::{PlayerId, RequestId};

#[cfg(feature = "serde")]
fn serialize_playerids<const N: usize, S>(
    t: &[PlayerId; N],
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    use serde::ser::SerializeTuple;

    let mut ser_tuple = serializer.serialize_tuple(N)?;
    for elem in t {
        ser_tuple.serialize_element(elem)?;
    }
    ser_tuple.end()
}

#[binrw]
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// Reorder the players
pub struct Reo {
    /// Non-zero if the packet is a packet request or a reply to a request
    pub reqi: RequestId,

    /// Number of players
    pub nump: u8,

    /// Order the players
    #[cfg_attr(feature = "serde", serde(serialize_with = "serialize_playerids"))]
    pub plid: [PlayerId; 40],
}

impl Default for Reo {
    fn default() -> Self {
        Self {
            reqi: RequestId(0),
            nump: 0,
            plid: [PlayerId(0); 40],
        }
    }
}

impl_typical_with_request_id!(Reo);
