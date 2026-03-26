use insim_core::{Decode, DecodeContext, Encode, EncodeContext};

use crate::identifiers::{PlayerId, RequestId};

const PLH_MAX_PLAYERS: usize = 48;

bitflags::bitflags! {
    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy, Default)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    /// Flags to indicate which handicap fields are set.
    pub struct PlayerHandicapFlags: u8 {
         const MASS = (1 << 0);
         const TRES = (1 << 1);
         const SILENT = (1 << 7);
    }
}

impl_bitflags_from_to_bytes!(PlayerHandicapFlags, u8);

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// Handicap settings for a single player.
pub struct PlayerHandicap {
    /// Player identifier.
    pub plid: PlayerId,

    /// Which handicap values are set.
    pub flags: PlayerHandicapFlags,

    /// Added mass (requires MASS flag).
    pub h_mass: u8,

    /// Intake restriction (requires TRES flag).
    pub h_tres: u8,
}

impl Decode for PlayerHandicap {
    fn decode(ctx: &mut DecodeContext) -> Result<Self, insim_core::DecodeError> {
        let plid = ctx.decode::<PlayerId>("plid")?;
        let flags = ctx.decode::<PlayerHandicapFlags>("flags")?;
        let h_mass = ctx.decode::<u8>("h_mass")?;
        let h_tres = ctx.decode::<u8>("h_tres")?;

        Ok(Self {
            plid,
            flags,
            h_mass,
            h_tres,
        })
    }
}

impl Encode for PlayerHandicap {
    fn encode(&self, ctx: &mut EncodeContext) -> Result<(), insim_core::EncodeError> {
        if self.h_mass > 200 {
            return Err(insim_core::EncodeErrorKind::OutOfRange {
                min: 0,
                max: 200,
                found: self.h_mass as usize,
            }
            .context("PlayerHandicap::h_mass"));
        }
        if self.h_tres > 50 {
            return Err(insim_core::EncodeErrorKind::OutOfRange {
                min: 0,
                max: 50,
                found: self.h_tres as usize,
            }
            .context("PlayerHandicap::h_tres"));
        }

        ctx.encode("plid", &self.plid)?;
        ctx.encode("flags", &self.flags)?;
        ctx.encode("h_mass", &self.h_mass)?;
        ctx.encode("h_tres", &self.h_tres)?;
        Ok(())
    }
}

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// Player handicap updates.
///
/// - Sets or reports per-player handicaps.
pub struct Plh {
    /// Request identifier echoed by replies.
    pub reqi: RequestId,

    /// Handicap list by player.
    pub hcaps: Vec<PlayerHandicap>,
}

impl_typical_with_request_id!(Plh);

impl Decode for Plh {
    fn decode(ctx: &mut DecodeContext) -> Result<Self, insim_core::DecodeError> {
        let reqi = ctx.decode::<RequestId>("reqi")?;
        let mut nump = ctx.decode::<u8>("nump")?;
        let mut hcaps = Vec::with_capacity(nump as usize);
        while nump > 0 {
            hcaps.push(ctx.decode::<PlayerHandicap>("hcaps")?);
            nump -= 1;
        }

        Ok(Self { reqi, hcaps })
    }
}

impl Encode for Plh {
    fn encode(&self, ctx: &mut EncodeContext) -> Result<(), insim_core::EncodeError> {
        ctx.encode("reqi", &self.reqi)?;
        let nump = self.hcaps.len();
        if nump > PLH_MAX_PLAYERS {
            return Err(insim_core::EncodeErrorKind::OutOfRange {
                min: 0,
                max: PLH_MAX_PLAYERS,
                found: nump,
            }
            .context("Plh handicaps out of range"));
        }
        ctx.encode("nump", &(nump as u8))?;
        for i in self.hcaps.iter() {
            ctx.encode("hcaps", i)?;
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
