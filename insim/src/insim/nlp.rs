use insim_core::{Decode, Encode};

use crate::identifiers::{PlayerId, RequestId};

#[derive(Debug, Clone, Default, insim_core::Decode, insim_core::Encode)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// Information about a specific vehicle/player. Used within [Nlp].
pub struct NodeLapInfo {
    /// Current path node
    pub node: u16,

    /// Current lap
    pub lap: u16,

    /// Player's unique ID
    pub plid: PlayerId,

    /// Player's race position
    pub position: u8,
}

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// Node and Lap packet - similar to Mci without positional information
pub struct Nlp {
    /// Non-zero if the packet is a packet request or a reply to a request
    pub reqi: RequestId,

    /// Node, lap and position of each player.
    pub info: Vec<NodeLapInfo>,
}

impl Decode for Nlp {
    fn decode(buf: &mut bytes::Bytes) -> Result<Self, insim_core::DecodeError> {
        let reqi = RequestId::decode(buf)?;
        let mut nump = u8::decode(buf)?;
        let mut info = Vec::with_capacity(nump as usize);
        while nump > 0 {
            info.push(NodeLapInfo::decode(buf)?);
            nump -= 1;
        }
        Ok(Self { reqi, info })
    }
}

impl Encode for Nlp {
    fn encode(&self, buf: &mut bytes::BytesMut) -> Result<(), insim_core::EncodeError> {
        self.reqi.encode(buf)?;
        let nump = self.info.len();
        if nump > 255 {
            return Err(insim_core::EncodeError::TooLarge);
        }
        (nump as u8).encode(buf)?;
        for i in self.info.iter() {
            i.encode(buf)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_nlp() {
        assert_from_to_bytes!(
            Nlp,
            [
                1,  // reqi
                2,  // nump
                26, // info[1] - node (1)
                1,  // info[1] - node (2)
                14, // info[1] - lap (1)
                0,  // info[1] - lap (2)
                5,  // info[1] - plid
                18, // info[1] - position
                35, // info[2] - node (1)
                5,  // info[2] - node (2)
                13, // info[2] - lap (1)
                0,  // info[2] - lap (2)
                6,  // info[2] - plid
                19, // info[2] - position
            ],
            |nlp: Nlp| {
                assert_eq!(nlp.reqi, RequestId(1));
                assert_eq!(nlp.info.len(), 2);
            }
        );
    }
}
