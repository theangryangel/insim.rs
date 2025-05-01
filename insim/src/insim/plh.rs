use insim_core::{Decode, Encode};

use crate::identifiers::{PlayerId, RequestId};

const PLH_MAX_PLAYERS: usize = 40;

bitflags::bitflags! {
    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy, Default)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize))]
    /// Flags to indicate which handicap(s) to set.
    pub struct PlayerHandicapFlags: u8 {
         const MASS = (1 << 0);
         const TRES = (1 << 1);
         const SILENT = (1 << 7);
    }
}

impl_bitflags_from_to_bytes!(PlayerHandicapFlags, u8);

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// Set the handicaps for a given player
pub struct PlayerHandicap {
    /// Player's unique ID
    pub plid: PlayerId,

    /// Handicaps values to set
    pub flags: PlayerHandicapFlags,

    /// Additional mass. Ensure that the flag `SET_MASS` is enabled.
    pub h_mass: u8,

    /// Additional intake restriction. Ensure that the flag `SET_TRES` is enabled
    pub h_tres: u8,
}

impl Decode for PlayerHandicap {
    fn decode(buf: &mut bytes::Bytes) -> Result<Self, insim_core::Error> {
        let plid = PlayerId::decode(buf)?;
        let flags = PlayerHandicapFlags::decode(buf)?;
        let h_mass = u8::decode(buf)?;
        let h_tres = u8::decode(buf)?;

        Ok(Self {
            plid,
            flags,
            h_mass,
            h_tres,
        })
    }
}

impl Encode for PlayerHandicap {
    fn encode(&self, buf: &mut bytes::BytesMut) -> Result<(), insim_core::Error> {
        if self.h_mass > 200 {
            return Err(insim_core::Error::TooLarge);
        }
        if self.h_tres > 50 {
            return Err(insim_core::Error::TooLarge);
        }

        self.plid.encode(buf)?;
        self.flags.encode(buf)?;
        self.h_mass.encode(buf)?;
        self.h_tres.encode(buf)?;
        Ok(())
    }
}

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// Player handicaps
pub struct Plh {
    /// Non-zero if the packet is a packet request or a reply to a request
    pub reqi: RequestId,

    /// List of handicaps by player
    pub hcaps: Vec<PlayerHandicap>,
}

impl_typical_with_request_id!(Plh);

impl Decode for Plh {
    fn decode(buf: &mut bytes::Bytes) -> Result<Self, insim_core::Error> {
        let reqi = RequestId::decode(buf)?;
        let mut nump = u8::decode(buf)?;
        let mut hcaps = Vec::with_capacity(nump as usize);
        while nump > 0 {
            hcaps.push(PlayerHandicap::decode(buf)?);
            nump -= 1;
        }

        Ok(Self { reqi, hcaps })
    }
}

impl Encode for Plh {
    fn encode(&self, buf: &mut bytes::BytesMut) -> Result<(), insim_core::Error> {
        self.reqi.encode(buf)?;
        let nump = self.hcaps.len();
        if nump > PLH_MAX_PLAYERS {
            return Err(insim_core::Error::TooLarge);
        }
        (nump as u8).encode(buf)?;
        for i in self.hcaps.iter() {
            i.encode(buf)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plh() {
        let data = [
            1,   // reqi
            3,   // nump
            1,   // hcaps[1] - plid
            1,   // hcaps[1] - flags
            200, // hcaps[1] - h_mass
            0,   // hcaps[1] - h_tres
            2,   // hcaps[2] - plid
            2,   // hcaps[2] - flags
            0,   // hcaps[2] - h_mass
            40,  // hcaps[2] - h_tres
            3,   // hcaps[3] - plid
            131, // hcaps[3] - flags
            200, // hcaps[3] - h_mass
            40,  // hcaps[3] - h_tres
        ];

        assert_from_to_bytes!(Plh, data, |plh: Plh| {
            assert_eq!(plh.reqi, RequestId(1));
            assert_eq!(plh.hcaps.len(), 3);

            assert_eq!(plh.hcaps[0].plid, PlayerId(1));
            assert_eq!(plh.hcaps[0].h_mass, 200);
            assert_eq!(plh.hcaps[0].h_tres, 0);
            assert!(plh.hcaps[0].flags.contains(PlayerHandicapFlags::MASS));
            assert!(!plh.hcaps[0].flags.contains(PlayerHandicapFlags::TRES));
            assert!(!plh.hcaps[0].flags.contains(PlayerHandicapFlags::SILENT));

            assert_eq!(plh.hcaps[1].plid, PlayerId(2));
            assert_eq!(plh.hcaps[1].h_mass, 0);
            assert_eq!(plh.hcaps[1].h_tres, 40);
            assert!(!plh.hcaps[1].flags.contains(PlayerHandicapFlags::MASS));
            assert!(plh.hcaps[1].flags.contains(PlayerHandicapFlags::TRES));
            assert!(!plh.hcaps[1].flags.contains(PlayerHandicapFlags::SILENT));

            assert_eq!(plh.hcaps[2].plid, PlayerId(3));
            assert_eq!(plh.hcaps[2].h_mass, 200);
            assert_eq!(plh.hcaps[2].h_tres, 40);

            assert!(plh.hcaps[2].flags.contains(PlayerHandicapFlags::MASS));
            assert!(plh.hcaps[2].flags.contains(PlayerHandicapFlags::TRES));
            assert!(plh.hcaps[2].flags.contains(PlayerHandicapFlags::SILENT));
        });
    }
}
