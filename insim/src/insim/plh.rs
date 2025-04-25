use insim_core::{
    binrw::{self, binrw},
    ReadWriteBuf,
};

use crate::identifiers::{PlayerId, RequestId};

const PLH_MAX_PLAYERS: usize = 40;

bitflags::bitflags! {
    #[binrw]
    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy, Default)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize))]
    #[br(map = Self::from_bits_truncate)]
    #[bw(map = |&x: &Self| x.bits())]
    /// Flags to indicate which handicap(s) to set.
    pub struct PlayerHandicapFlags: u8 {
         const MASS = (1 << 0);
         const TRES = (1 << 1);
         const SILENT = (1 << 7);
    }
}

impl_bitflags_from_to_bytes!(PlayerHandicapFlags, u8);

#[binrw]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[bw(assert(*h_mass <= 200))]
#[bw(assert(*h_tres <= 50))]
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

impl ReadWriteBuf for PlayerHandicap {
    fn read_buf(buf: &mut bytes::Bytes) -> Result<Self, insim_core::Error> {
        let plid = PlayerId::read_buf(buf)?;
        let flags = PlayerHandicapFlags::read_buf(buf)?;
        let h_mass = u8::read_buf(buf)?;
        let h_tres = u8::read_buf(buf)?;

        Ok(Self {
            plid,
            flags,
            h_mass,
            h_tres,
        })
    }

    fn write_buf(&self, buf: &mut bytes::BytesMut) -> Result<(), insim_core::Error> {
        if self.h_mass > 200 {
            return Err(insim_core::Error::TooLarge);
        }
        if self.h_tres > 50 {
            return Err(insim_core::Error::TooLarge);
        }

        self.plid.write_buf(buf)?;
        self.flags.write_buf(buf)?;
        self.h_mass.write_buf(buf)?;
        self.h_tres.write_buf(buf)?;
        Ok(())
    }
}

#[binrw]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// Player handicaps
pub struct Plh {
    /// Non-zero if the packet is a packet request or a reply to a request
    pub reqi: RequestId,

    #[bw(calc = hcaps.len() as u8)]
    nump: u8,

    /// List of handicaps by player
    #[br(count = nump)]
    pub hcaps: Vec<PlayerHandicap>,
}

impl_typical_with_request_id!(Plh);

impl ReadWriteBuf for Plh {
    fn read_buf(buf: &mut bytes::Bytes) -> Result<Self, insim_core::Error> {
        let reqi = RequestId::read_buf(buf)?;
        let mut nump = u8::read_buf(buf)?;
        let mut hcaps = Vec::with_capacity(nump as usize);
        while nump > 0 {
            hcaps.push(PlayerHandicap::read_buf(buf)?);
            nump -= 1;
        }

        Ok(Self { reqi, hcaps })
    }

    fn write_buf(&self, buf: &mut bytes::BytesMut) -> Result<(), insim_core::Error> {
        self.reqi.write_buf(buf)?;
        let nump = self.hcaps.len();
        if nump > PLH_MAX_PLAYERS {
            return Err(insim_core::Error::TooLarge);
        }
        (nump as u8).write_buf(buf)?;
        for i in self.hcaps.iter() {
            i.write_buf(buf)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use binrw::{BinRead, BinWrite};

    use super::*;

    #[test]
    fn test_plh() {
        let data = [
            1,   // ReqI
            3,   // NumP
            1,   // HCaps[1] - PLID
            1,   // HCaps[1] - Flags
            200, // HCaps[1] - H_Mass
            0,   // HCaps[1] - H_TRes
            2,   // HCaps[2] - PLID
            2,   // HCaps[2] - Flags
            0,   // HCaps[2] - H_Mass
            40,  // HCaps[2] - H_TRes
            3,   // HCaps[3] - PLID
            131, // HCaps[3] - Flags
            200, // HCaps[3] - H_Mass
            40,  // HCaps[3] - H_TRes
        ];

        let mut c = Cursor::new(&data);
        let plh = Plh::read_le(&mut c).unwrap();

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

        let mut output = Cursor::new(Vec::new());
        plh.write_le(&mut output).unwrap();
        let output = output.into_inner();

        assert_eq!(&output, &data);
    }
}
