use insim_core::{
    binrw::{self, binrw},
    identifiers::{PlayerId, RequestId},
};

#[cfg(feature = "serde")]
use serde::Serialize;

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
#[cfg_attr(feature = "serde", derive(Serialize))]
/// Reorder
pub struct Reo {
    pub reqi: RequestId,
    pub nump: u8,

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
