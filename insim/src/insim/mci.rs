use bitflags::bitflags;
use bytes::{Buf, BufMut};
use insim_core::{direction::Direction, point::Point, speed::Speed, Decode, Encode};

use crate::identifiers::{PlayerId, RequestId};

bitflags! {
    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy, Default)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize))]
    /// Additional Car Info.
    pub struct CompCarInfo: u8 {
        /// This car is in the way of a driver who is a lap ahead
        const BLUE = (1 << 0);

        /// This car is slow or stopped and in a dangerous place
        const YELLOW = (1 << 1);

        /// This car is lagging (missing or delayed position packets)
        const LAG = (1 << 5);

        /// This is the first compcar in this set of MCI packets
        const FIRST = (1 << 6);

        /// This is the last compcar in this set of MCI packets
        const LAST = (1 << 7);
    }
}

generate_bitflag_helpers! {
    CompCarInfo,

    pub has_blue_flag => BLUE,
    pub has_yellow_flag => YELLOW,
    pub is_lagging => LAG,
    pub is_first => FIRST,
    pub is_last => LAST
}

impl_bitflags_from_to_bytes!(CompCarInfo, u8);

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// Used within the [Mci] packet info field.
pub struct CompCar {
    /// Index of the last "node" that the player passed through.
    pub node: u16,

    /// The player's current lap.
    pub lap: u16,

    /// The current player's ID.
    pub plid: PlayerId,

    /// Race position
    pub position: u8,

    /// Additional information that describes this particular Compcar.
    pub info: CompCarInfo,

    /// Positional information for the player, in game units.
    pub xyz: Point<i32>,

    /// Speed
    pub speed: Speed,

    /// Direction of car's motion : 0 = world y direction, 32768 = 180 deg
    pub direction: Direction,

    /// Direction of forward axis : 0 = world y direction, 32768 = 180 deg
    pub heading: Direction,

    /// Signed, rate of change of heading : (16384 = 360 deg/s)
    // FIXME strongly type
    pub angvel: i16,
}

impl CompCar {
    /// This is the first compcar in this set of MCI packets
    pub fn is_first(&self) -> bool {
        self.info.is_first()
    }

    /// This is the last compcar in this set of MCI packets
    pub fn is_last(&self) -> bool {
        self.info.is_last()
    }
}

impl Decode for CompCar {
    fn decode(buf: &mut bytes::Bytes) -> Result<Self, insim_core::DecodeError> {
        let node = u16::decode(buf)?;
        let lap = u16::decode(buf)?;
        let plid = PlayerId::decode(buf)?;
        let position = u8::decode(buf)?;
        let info = CompCarInfo::decode(buf)?;
        buf.advance(1);
        let xyz = Point::<i32>::decode(buf)?;
        let speed = Speed::from_game_mci_units(u16::decode(buf)?);
        let direction = Direction::from_game_units(u16::decode(buf)?);
        let heading = Direction::from_game_units(u16::decode(buf)?);
        let angvel = i16::decode(buf)?;
        Ok(Self {
            node,
            lap,
            plid,
            position,
            info,
            xyz,
            speed,
            direction,
            heading,
            angvel,
        })
    }
}

impl Encode for CompCar {
    fn encode(&self, buf: &mut bytes::BytesMut) -> Result<(), insim_core::EncodeError> {
        self.node.encode(buf)?;
        self.lap.encode(buf)?;
        self.plid.encode(buf)?;
        self.position.encode(buf)?;
        self.info.encode(buf)?;
        buf.put_bytes(0, 1);
        self.xyz.encode(buf)?;
        self.speed.as_game_mci_units().encode(buf)?;
        self.direction.as_game_units().encode(buf)?;
        self.heading.as_game_units().encode(buf)?;
        self.angvel.encode(buf)?;
        Ok(())
    }
}

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// Multi Car Info - positional information for players/vehicles.
/// The MCI packet does not contain the positional information for all players. Only some. The
/// maximum number of players depends on the version of Insim.
pub struct Mci {
    /// Non-zero if the packet is a packet request or a reply to a request
    pub reqi: RequestId,

    /// Node and lap for a subset of players. Not all players may be included in a single packet.
    pub info: Vec<CompCar>,
}

impl Mci {
    /// Is this the first MCI packet in this set of MCI packets?
    pub fn is_first(&self) -> bool {
        self.info.iter().any(|i| i.is_first())
    }

    /// Is this the last MCI packet in this set of MCI packets?
    pub fn is_last(&self) -> bool {
        self.info.iter().any(|i| i.is_last())
    }
}

impl Decode for Mci {
    fn decode(buf: &mut bytes::Bytes) -> Result<Self, insim_core::DecodeError> {
        let reqi = RequestId::decode(buf)?;
        let mut numc = u8::decode(buf)?;
        let mut info = Vec::with_capacity(numc as usize);
        while numc > 0 {
            info.push(CompCar::decode(buf)?);
            numc -= 1;
        }
        Ok(Self { reqi, info })
    }
}

impl Encode for Mci {
    fn encode(&self, buf: &mut bytes::BytesMut) -> Result<(), insim_core::EncodeError> {
        self.reqi.encode(buf)?;
        let numc = self.info.len();
        if numc > 255 {
            return Err(insim_core::EncodeError::TooLarge);
        }
        (numc as u8).encode(buf)?;
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
    fn test_mci() {
        assert_from_to_bytes!(
            Mci,
            [
                0,   // reqi
                2,   // numc
                57,  // info[0] - node (1)
                0,   // info[0] - node (2)
                1,   // info[0] - lap (1)
                0,   // info[0] - lap (1)
                17,  // info[0] - plid
                1,   // info[0] - position
                64,  // info[0] - info
                0,   // info[0] - sp3
                107, // info[0] - x (1)
                112, // info[0] - x (2)
                252, // info[0] - x (3)
                0,   // info[0] - x (4)
                142, // info[0] - y (1)
                220, // info[0] - y (2)
                71,  // info[0] - y (3)
                0,   // info[0] - y (4)
                18,  // info[0] - z (1)
                223, // info[0] - z (2)
                3,   // info[0] - z (3)
                0,   // info[0] - z (4)
                147, // info[0] - speed (1)
                14,  // info[0] - speed (2)
                254, // info[0] - direction (1)
                222, // info[0] - direction (2)
                16,  // info[0] - heading (1)
                223, // info[0] - heading (2)
                192, // info[0] - angvel (1)
                255, // info[0] - angvel (2)
                60,  // info[1] - node (1)
                0,   // info[1] - node (2)
                1,   // info[1] - lap (1)
                0,   // info[1] - lap (1)
                18,  // info[1] - plid
                2,   // info[1] - position
                128, // info[1] - info
                0,   // info[1] - sp3
                193, // info[1] - x (1)
                48,  // info[1] - x (2)
                14,  // info[1] - x (3)
                1,   // info[1] - x (4)
                237, // info[1] - y (1)
                94,  // info[1] - y (2)
                81,  // info[1] - y (3)
                0,   // info[1] - y (4)
                211, // info[1] - z (1)
                131, // info[1] - z (2)
                3,   // info[1] - z (3)
                0,   // info[1] - z (4)
                224, // info[1] - speed (1)
                17,  // info[1] - speed (2)
                36,  // info[1] - direction (1)
                220, // info[1] - direction (2)
                250, // info[1] - heading (1)
                219, // info[1] - heading (2)
                71,  // info[1] - angvel (1)
                0,   // info[1] - angvel (2)
            ],
            |mci: Mci| {
                assert_eq!(mci.reqi, RequestId(0));
                assert_eq!(mci.info.len(), 2);
            }
        );
    }
}
