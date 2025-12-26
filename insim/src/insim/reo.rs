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

const REO_MAX_PLAYERS: usize = 48;

#[derive(Debug, Clone, insim_core::Decode, insim_core::Encode)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// Reorder the players
pub struct Reo {
    /// Non-zero if the packet is a packet request or a reply to a request
    pub reqi: RequestId,

    /// Number of players
    pub nump: u8,

    /// Order the players
    #[cfg_attr(feature = "serde", serde(serialize_with = "serialize_playerids"))]
    pub plid: [PlayerId; REO_MAX_PLAYERS],
}

impl Default for Reo {
    fn default() -> Self {
        Self {
            reqi: RequestId(0),
            nump: 0,
            plid: [PlayerId(0); REO_MAX_PLAYERS],
        }
    }
}

impl_typical_with_request_id!(Reo);

#[cfg(test)]
mod test {
    use bytes::{BufMut, BytesMut};

    use super::*;

    #[test]
    fn test_reo() {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(&[0, REO_MAX_PLAYERS as u8]);
        for i in 0..REO_MAX_PLAYERS as u8 {
            buf.put_u8(i);
        }

        assert_from_to_bytes!(Reo, buf.as_ref(), |parsed: Reo| {
            assert_eq!(parsed.reqi, RequestId(0));
            assert_eq!(parsed.nump, REO_MAX_PLAYERS as u8);
            for i in 0..40 {
                assert_eq!(parsed.plid[i], PlayerId(i as u8));
            }
        });
    }
}
