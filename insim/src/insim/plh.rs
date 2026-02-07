use insim_core::{Decode, Encode};

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
    fn decode(buf: &mut bytes::Bytes) -> Result<Self, insim_core::DecodeError> {
        let plid = PlayerId::decode(buf).map_err(|e| e.nested().context("PlayerHandicap::plid"))?;
        let flags = PlayerHandicapFlags::decode(buf)
            .map_err(|e| e.nested().context("PlayerHandicap::flags"))?;
        let h_mass = u8::decode(buf).map_err(|e| e.nested().context("PlayerHandicap::h_mass"))?;
        let h_tres = u8::decode(buf).map_err(|e| e.nested().context("PlayerHandicap::h_tres"))?;

        Ok(Self {
            plid,
            flags,
            h_mass,
            h_tres,
        })
    }
}

impl Encode for PlayerHandicap {
    fn encode(&self, buf: &mut bytes::BytesMut) -> Result<(), insim_core::EncodeError> {
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

        self.plid
            .encode(buf)
            .map_err(|e| e.nested().context("PlayerHandicap::plid"))?;
        self.flags
            .encode(buf)
            .map_err(|e| e.nested().context("PlayerHandicap::flags"))?;
        self.h_mass
            .encode(buf)
            .map_err(|e| e.nested().context("PlayerHandicap::h_mass"))?;
        self.h_tres
            .encode(buf)
            .map_err(|e| e.nested().context("PlayerHandicap::h_tres"))?;
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
    fn decode(buf: &mut bytes::Bytes) -> Result<Self, insim_core::DecodeError> {
        let reqi = RequestId::decode(buf).map_err(|e| e.nested().context("Plh::reqi"))?;
        let mut nump = u8::decode(buf).map_err(|e| e.nested().context("Plh::nump"))?;
        let mut hcaps = Vec::with_capacity(nump as usize);
        while nump > 0 {
            hcaps.push(PlayerHandicap::decode(buf).map_err(|e| e.nested().context("Plh::hcaps"))?);
            nump -= 1;
        }

        Ok(Self { reqi, hcaps })
    }
}

impl Encode for Plh {
    fn encode(&self, buf: &mut bytes::BytesMut) -> Result<(), insim_core::EncodeError> {
        self.reqi
            .encode(buf)
            .map_err(|e| e.nested().context("Plh::reqi"))?;
        let nump = self.hcaps.len();
        if nump > PLH_MAX_PLAYERS {
            return Err(insim_core::EncodeErrorKind::OutOfRange {
                min: 0,
                max: PLH_MAX_PLAYERS,
                found: nump,
            }
            .context("Plh handicaps out of range"));
        }
        (nump as u8)
            .encode(buf)
            .map_err(|e| e.nested().context("Plh::nump"))?;
        for i in self.hcaps.iter() {
            i.encode(buf)
                .map_err(|e| e.nested().context("Plh::hcaps"))?;
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
