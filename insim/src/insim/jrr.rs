use bytes::BufMut;
use insim_core::{Decode, Encode, heading::Heading, object::ObjectCoordinate};

use crate::identifiers::{ConnectionId, PlayerId, RequestId};

#[derive(Debug, Default, Clone, insim_core::Decode, insim_core::Encode)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(u8)]
#[non_exhaustive]
/// Used within the [Jrr] packet.
pub enum JrrAction {
    #[default]
    /// Reject the join request
    Reject = 0,

    /// Allow the user to spawn
    Spawn = 1,

    /// Move the player
    Reset = 4,

    /// Move the player, but do not repair
    ResetNoRepair = 5,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum JrrStartPosition {
    DefaultStartPosition,
    Custom {
        /// Position
        xyz: ObjectCoordinate,
        /// Heading / Direction
        heading: Heading,
    }
}

impl Decode for JrrStartPosition {
    fn decode(buf: &mut bytes::Bytes) -> Result<Self, insim_core::DecodeError> {
        let x = i16::decode(buf)?;
        let y = i16::decode(buf)?;
        let z = u8::decode(buf)?;

        let flags = u8::decode(buf)?;
        let index = u8::decode(buf)?;
        let heading = u8::decode(buf)?;

        if x == 0 && y == 0 && z == 8 && flags == 0 && index == 0 && heading == 0 {
            Ok(Self::DefaultStartPosition)
        } else {
            Ok(Self::Custom { xyz: ObjectCoordinate {
                x, y, z
            }, heading: Heading::from_objectinfo_wire(heading) })
        }
    }
}

impl Encode for JrrStartPosition {
    fn encode(&self, buf: &mut bytes::BytesMut) -> Result<(), insim_core::EncodeError> {
        match self {
            JrrStartPosition::DefaultStartPosition => buf.put_bytes(0, 8),
            JrrStartPosition::Custom { xyz, heading } => {
                xyz.x.encode(buf)?;
                xyz.y.encode(buf)?;
                xyz.z.encode(buf)?;
                0x80u8.encode(buf)?; // flags
                buf.put_u8(0); // index
                heading.to_objectinfo_wire().encode(buf)?;
            },
        };
        Ok(())
    }
}

#[derive(Debug, Clone, insim_core::Decode, insim_core::Encode)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// Join Request Reply
/// Set the ISF_REQ_JOIN flag in the IS_ISI to receive join requests
///
/// A join request is seen as an IS_NPL packet with ZERO in the NumP field
/// An immediate response (e.g. within 1 second) is required using an IS_JRR packet
/// In this case, PLID must be zero and JRRAction must be JRR_REJECT or JRR_SPAWN
/// If you allow the join and it is successful you will then get a normal IS_NPL with NumP set
/// You can also specify the start position of the car using the StartPos structure
///
/// IS_JRR can also be used to move an existing car to a different location
/// In this case, PLID must be set, JRRAction must be JRR_RESET or higher and StartPos must be set
pub struct Jrr {
    /// Non-zero if the packet is a packet request or a reply to a request
    pub reqi: RequestId,

    /// Player's unique ID
    pub plid: PlayerId,

    /// Unique connection ID
    pub ucid: ConnectionId,

    #[insim(pad_after = 2)]
    /// Action taken/to take
    pub jrraction: JrrAction,

    /// Start position
    pub startpos: JrrStartPosition,
}

impl_typical_with_request_id!(Jrr);

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_jrr() {
        assert_from_to_bytes!(
            Jrr,
            [
                0,   // reqi
                3,   // plid
                2,   // ucid
                1,   // jrraction
                0,   // sp2
                0,   // sp3
                172, // startpos - x (1)
                218, // startpos - x (2)
                25,  // startpos - y (1)
                136, // startpos - y (2)
                12,  // startpos - zbyte (1)
                128, // startpos - flags
                0,   // startpos - index
                67,  // startpos - heading
            ],
            |jrr: Jrr| {
                assert_eq!(jrr.reqi, RequestId(0));
                assert!(matches!(jrr.jrraction, JrrAction::Spawn));
                assert!(matches!(
                    jrr.startpos,
                    JrrStartPosition::Custom {
                        xyz: ObjectCoordinate {
                            x: -9556,
                            y: -30695,
                            z: 12,
                        },
                        ..
                    }
                ));
            }
        );
    }
}
