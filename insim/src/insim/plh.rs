use insim_core::{
    binrw::{self, binrw},
    identifiers::{PlayerId, RequestId},
};

#[cfg(feature = "serde")]
use serde::Serialize;

bitflags::bitflags! {
    #[binrw]
    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy, Default)]
    #[cfg_attr(feature = "serde", derive(Serialize))]
    #[br(map = Self::from_bits_truncate)]
    #[bw(map = |&x: &Self| x.bits())]
    pub struct PlayerHandicapFlags: u8 {
         const SET_MASS = (1 << 0);
         const SET_TRES = (1 << 1);
         const SILENT = (1 << 7);
    }
}

#[binrw]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[bw(assert(*h_mass <= 200))]
#[bw(assert(*h_tres <= 50))]
pub struct PlayerHandicap {
    pub plid: PlayerId,
    pub flags: PlayerHandicapFlags,
    pub h_mass: u8,
    pub h_tres: u8,
}

#[binrw]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
/// Player handicaps
pub struct Plh {
    pub reqi: RequestId,

    #[bw(calc = hcap.len() as u8)]
    nump: u8,

    #[br(count = nump)]
    pub hcap: Vec<PlayerHandicap>,
}

#[cfg(test)]
mod tests {
    use binrw::{BinRead, BinWrite};
    use std::io::Cursor;

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
        assert_eq!(plh.hcap.len(), 3);

        assert_eq!(plh.hcap[0].plid, PlayerId(1));
        assert_eq!(plh.hcap[0].h_mass, 200);
        assert_eq!(plh.hcap[0].h_tres, 0);
        assert!(plh.hcap[0].flags.contains(PlayerHandicapFlags::SET_MASS));
        assert!(!plh.hcap[0].flags.contains(PlayerHandicapFlags::SET_TRES));
        assert!(!plh.hcap[0].flags.contains(PlayerHandicapFlags::SILENT));

        assert_eq!(plh.hcap[1].plid, PlayerId(2));
        assert_eq!(plh.hcap[1].h_mass, 0);
        assert_eq!(plh.hcap[1].h_tres, 40);
        assert!(!plh.hcap[1].flags.contains(PlayerHandicapFlags::SET_MASS));
        assert!(plh.hcap[1].flags.contains(PlayerHandicapFlags::SET_TRES));
        assert!(!plh.hcap[1].flags.contains(PlayerHandicapFlags::SILENT));

        assert_eq!(plh.hcap[2].plid, PlayerId(3));
        assert_eq!(plh.hcap[2].h_mass, 200);
        assert_eq!(plh.hcap[2].h_tres, 40);

        assert!(plh.hcap[2].flags.contains(PlayerHandicapFlags::SET_MASS));
        assert!(plh.hcap[2].flags.contains(PlayerHandicapFlags::SET_TRES));
        assert!(plh.hcap[2].flags.contains(PlayerHandicapFlags::SILENT));

        let mut output = Cursor::new(Vec::new());
        plh.write_le(&mut output).unwrap();
        let output = output.into_inner();

        assert_eq!(&output, &data);
    }
}
