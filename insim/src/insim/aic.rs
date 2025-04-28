use std::time::Duration;

use bitflags::bitflags;
use bytes::Buf;
use insim_core::ReadWriteBuf;

use crate::identifiers::{PlayerId, RequestId};

const AIC_MAX_INPUTS: usize = 20;

bitflags! {
    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy, Default)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize))]
    /// Describes the setup of a player and the various helpers that may be enabled, such as
    /// auto-clutch, etc.
    pub struct AiHelpFlags: u16 {
        /// Autogears
        const AUTOGEARS = (1 << 3);
        /// Brake help
        const HELP_B = (1 << 6);
        /// Autoclutch
        const AUTOCLUTCH = (1 << 9);
    }
}

impl_bitflags_from_to_bytes!(AiHelpFlags, u16);

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[non_exhaustive]
/// AI input type
// FIXME: Strongly type!
pub enum AiInputType {
    /// Steering
    Msx(u16),

    /// Throttle
    Throttle(u16),

    /// Brake
    Brake(u16),

    /// Gear up
    Chup(u16),

    /// Gear down
    Chdn(u16),

    /// Ignition
    Ignition(u16),

    /// Extra lights
    ExtraLight(u16),

    /// Head lights
    HeadLights(u16),

    /// Siren
    Siren(u16),

    /// Honk
    Horn(u16),

    /// Flash
    Flash(u16),

    /// Clutch
    Clutch(u16),

    /// Handbrake
    Handbrake(u16),

    /// Indicators
    Indicators(u16),

    /// Gear
    Gear(u16),

    /// Look
    Look(u16),

    /// Pitspeed
    Pitspeed(u16),

    /// Disable Traction Control
    TcDisable(u16),

    /// Fogs rear
    FogRear(u16),

    /// Fogs front
    FogFront(u16),

    /// Send AI Info
    SendAiInfo,

    // Repeat AI Information at a given interval. 0 to stop.
    RepeatAiInfo(Duration),

    // Set help flags
    SetHelpFlags(AiHelpFlags),

    /// Reset all controlled values
    ResetAll,

    /// Yield control
    StopControl,
}

impl Default for AiInputType {
    fn default() -> Self {
        Self::Msx(32768)
    }
}

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// AI Input Control, value
pub struct AiInputVal {
    /// Input
    pub input: AiInputType,

    /// Duration
    pub time: Duration,
}

impl ReadWriteBuf for AiInputVal {
    fn read_buf(buf: &mut bytes::Bytes) -> Result<Self, insim_core::Error> {
        let input = u8::read_buf(buf)?;
        let time = u8::read_buf(buf)?;
        let val = u16::read_buf(buf)?;

        let input = match input {
            0 => AiInputType::Msx(val),
            1 => AiInputType::Throttle(val),
            2 => AiInputType::Brake(val),
            3 => AiInputType::Chup(val),
            4 => AiInputType::Chdn(val),
            5 => AiInputType::Ignition(val),
            6 => AiInputType::ExtraLight(val),
            7 => AiInputType::HeadLights(val),
            8 => AiInputType::Siren(val),
            9 => AiInputType::Horn(val),
            10 => AiInputType::Flash(val),
            11 => AiInputType::Clutch(val),
            12 => AiInputType::Handbrake(val),
            13 => AiInputType::Indicators(val),
            14 => AiInputType::Gear(val),
            15 => AiInputType::Look(val),
            16 => AiInputType::Pitspeed(val),
            17 => AiInputType::TcDisable(val),
            18 => AiInputType::FogRear(val),
            19 => AiInputType::FogFront(val),
            240 => AiInputType::SendAiInfo,
            241 => AiInputType::RepeatAiInfo(Duration::from_millis(val as u64 * 10)),
            253 => {
                let flags = AiHelpFlags::from_bits_truncate(val);
                AiInputType::SetHelpFlags(flags)
            },
            254 => AiInputType::ResetAll,
            255 => AiInputType::StopControl,
            found => {
                return Err(insim_core::Error::NoVariantMatch {
                    found: found as u64,
                })
            },
        };

        let time = Duration::from_millis(time as u64 * 10);

        Ok(Self { input, time })
    }

    fn write_buf(&self, buf: &mut bytes::BytesMut) -> Result<(), insim_core::Error> {
        let (discrim, val): (u8, u16) = match self.input {
            AiInputType::Msx(val) => (0, val),
            AiInputType::Throttle(val) => (1, val),
            AiInputType::Brake(val) => (2, val),
            AiInputType::Chup(val) => (3, val),
            AiInputType::Chdn(val) => (4, val),
            AiInputType::Ignition(val) => (5, val),
            AiInputType::ExtraLight(val) => (6, val),
            AiInputType::HeadLights(val) => (7, val),
            AiInputType::Siren(val) => (8, val),
            AiInputType::Horn(val) => (9, val),
            AiInputType::Flash(val) => (10, val),
            AiInputType::Clutch(val) => (11, val),
            AiInputType::Handbrake(val) => (12, val),
            AiInputType::Indicators(val) => (13, val),
            AiInputType::Gear(val) => (14, val),
            AiInputType::Look(val) => (15, val),
            AiInputType::Pitspeed(val) => (16, val),
            AiInputType::TcDisable(val) => (17, val),
            AiInputType::FogRear(val) => (18, val),
            AiInputType::FogFront(val) => (19, val),
            AiInputType::SendAiInfo => (240, 0),
            AiInputType::RepeatAiInfo(val) => {
                (
                    241,
                    // FIXME: check boundary
                    (((val.as_millis()) / 10) as u16),
                )
            },
            AiInputType::SetHelpFlags(val) => (253, val.bits()),
            AiInputType::ResetAll => (254, 0),
            AiInputType::StopControl => (255, 0),
        };

        discrim.write_buf(buf)?;

        // FIXME: check boundary
        let time = (self.time.as_millis() / 10) as u8;
        time.write_buf(buf)?;

        val.write_buf(buf)?;
        Ok(())
    }
}

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// AI Input Control
pub struct Aic {
    /// Non-zero if the packet is a packet request or a reply to a request
    pub reqi: RequestId,

    /// Set to choose 16-bit
    pub plid: PlayerId,

    /// Inputs to send
    pub inputs: Vec<AiInputVal>,
}

impl_typical_with_request_id!(Aic);

impl ReadWriteBuf for Aic {
    fn read_buf(buf: &mut bytes::Bytes) -> Result<Self, insim_core::Error> {
        let reqi = RequestId::read_buf(buf)?;
        let plid = PlayerId::read_buf(buf)?;
        let mut inputs = Vec::new();
        while buf.has_remaining() {
            inputs.push(AiInputVal::read_buf(buf)?);
        }

        Ok(Self { reqi, plid, inputs })
    }

    fn write_buf(&self, buf: &mut bytes::BytesMut) -> Result<(), insim_core::Error> {
        self.reqi.write_buf(buf)?;
        self.plid.write_buf(buf)?;
        if self.inputs.len() > AIC_MAX_INPUTS {
            return Err(insim_core::Error::TooLarge);
        }
        for i in self.inputs.iter() {
            i.write_buf(buf)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_aic() {
        assert_from_to_bytes!(
            Aic,
            [
                0, // reqi
                4, // plid
                // input 1
                15, // inputs[0].input - LOOK
                10, // inputs[0].time
                4,  // inputs[0].val[0] - LEFT
                0,  // inputs[0].val[1]
                // input 2
                13,  // inputs[0].input - INDICATORS
                100, // inputs[0].time
                4,   // inputs[0].val[0] - Hazards
                0,   // inputs[0].val[1]
            ],
            |parsed: Aic| {
                assert_eq!(parsed.reqi, RequestId(0));
                assert_eq!(parsed.plid, PlayerId(4));
                assert_eq!(parsed.inputs.len(), 2);
                assert!(matches!(parsed.inputs[0].input, AiInputType::Look(4)));
                assert!(matches!(parsed.inputs[1].input, AiInputType::Indicators(4)));
            }
        );
    }
}
