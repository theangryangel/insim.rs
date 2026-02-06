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

#[cfg(feature = "serde")]
fn deserialize_playerids<'de, const N: usize, D>(deserializer: D) -> Result<[PlayerId; N], D::Error>
where
    D: serde::Deserializer<'de>,
{
    struct ExpectedPlayerIdArray<const N: usize>;

    impl<const N: usize> serde::de::Expected for ExpectedPlayerIdArray<N> {
        fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            write!(formatter, "an array of {} PlayerId entries", N)
        }
    }

    let values: Vec<PlayerId> = serde::Deserialize::deserialize(deserializer)?;
    values.try_into().map_err(|values: Vec<PlayerId>| {
        serde::de::Error::invalid_length(values.len(), &ExpectedPlayerIdArray::<N>)
    })
}

const REO_MAX_PLAYERS: usize = 48;

#[derive(Debug, Clone, insim_core::Decode, insim_core::Encode)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// Starting order information or instruction.
///
/// - Sent at the start of a race or qualifying session.
/// - Can be requested via [`TinyType::Reo`](crate::insim::TinyType::Reo).
pub struct Reo {
    /// Request identifier echoed by replies.
    pub reqi: RequestId,

    /// Number of players in the order list.
    pub nump: u8,

    /// Player order (first `nump` entries are used).
    #[cfg_attr(
        feature = "serde",
        serde(
            serialize_with = "serialize_playerids",
            deserialize_with = "deserialize_playerids"
        )
    )]
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
