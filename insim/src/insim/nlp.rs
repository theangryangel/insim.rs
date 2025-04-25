use insim_core::{
    binrw::{self, binrw},
    ReadWriteBuf,
};

use crate::identifiers::{PlayerId, RequestId};

#[binrw]
#[derive(Debug, Clone, Default, insim_macros::ReadWriteBuf)]
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

#[binrw]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// Node and Lap packet - similar to Mci without positional information
pub struct Nlp {
    /// Non-zero if the packet is a packet request or a reply to a request
    pub reqi: RequestId,

    #[bw(calc = info.len() as u8)]
    nump: u8,

    /// Node, lap and position of each player.
    #[br(count = nump)]
    pub info: Vec<NodeLapInfo>,
}

impl ReadWriteBuf for Nlp {
    fn read_buf(buf: &mut bytes::Bytes) -> Result<Self, insim_core::Error> {
        let reqi = RequestId::read_buf(buf)?;
        let mut nump = u8::read_buf(buf)?;
        let mut info = Vec::with_capacity(nump as usize);
        while nump > 0 {
            info.push(NodeLapInfo::read_buf(buf)?);
            nump -= 1;
        }
        Ok(Self { reqi, info })
    }

    fn write_buf(&self, buf: &mut bytes::BytesMut) -> Result<(), insim_core::Error> {
        self.reqi.write_buf(buf)?;
        let nump = self.info.len();
        if nump > 255 {
            return Err(insim_core::Error::TooLarge);
        }
        (nump as u8).write_buf(buf)?;
        for i in self.info.iter() {
            i.write_buf(buf)?;
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
