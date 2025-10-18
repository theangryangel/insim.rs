use std::time::Duration;

use bitflags::bitflags;
use bytes::Buf;
use insim_core::{Decode, Encode};

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

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[non_exhaustive]
/// Special toggle-able helper for [AiInputType]
pub enum AiInputToggle {
    /// Toggle on/off
    #[default]
    Toggle = 1,
    /// Turn off
    Off = 2,
    /// Turn on
    On = 3,
}

impl From<u16> for AiInputToggle {
    fn from(value: u16) -> Self {
        match value {
            3 => Self::On,
            2 => Self::Off,
            _ => Self::Toggle,
        }
    }
}

impl From<AiInputToggle> for u16 {
    fn from(value: AiInputToggle) -> Self {
        match value {
            AiInputToggle::On => 3,
            AiInputToggle::Off => 2,
            AiInputToggle::Toggle => 1,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[non_exhaustive]
/// AI input type
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
    Ignition(AiInputToggle),

    /// Extra lights
    ExtraLight(AiInputToggle),

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
    Pitspeed(AiInputToggle),

    /// Disable Traction Control
    TcDisable(AiInputToggle),

    /// Fogs rear
    FogRear(AiInputToggle),

    /// Fogs front
    FogFront(AiInputToggle),

    /// Send AI Info
    SendAiInfo,

    /// Repeat AI Information at a given interval. 0 to stop.
    RepeatAiInfo,

    /// Set help flags
    SetHelpFlags(AiHelpFlags),

    /// Reset all controlled values
    ResetAll,

    /// Yield control
    StopControl,
}

impl Default for AiInputType {
    fn default() -> Self {
        Self::ResetAll
    }
}

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// AI Input Control, value
pub struct AiInput {
    /// Input
    pub input: AiInputType,

    /// Duration
    pub time: Option<Duration>,
}

impl AiInput {
    /// Headlights off
    pub fn headlights_off(time: Option<Duration>) -> Self {
        Self {
            input: AiInputType::HeadLights(1),
            time,
        }
    }

    /// Headlights side
    pub fn headlights_side(time: Option<Duration>) -> Self {
        Self {
            input: AiInputType::HeadLights(2),
            time,
        }
    }

    /// Headlights low
    pub fn headlights_low(time: Option<Duration>) -> Self {
        Self {
            input: AiInputType::HeadLights(3),
            time,
        }
    }

    /// HeadLights high
    pub fn headlights_high(time: Option<Duration>) -> Self {
        Self {
            input: AiInputType::HeadLights(4),
            time,
        }
    }

    /// Siren fast
    pub fn siren_fast(time: Option<Duration>) -> Self {
        Self {
            input: AiInputType::Siren(1),
            time,
        }
    }

    /// Siren low
    pub fn siren_slow(time: Option<Duration>) -> Self {
        Self {
            input: AiInputType::Siren(2),
            time,
        }
    }

    /// Horn: 1 to 5
    pub fn horn(level: u8, time: Option<Duration>) -> Self {
        Self {
            input: AiInputType::Horn(level as u16),
            time,
        }
    }

    /// Flash lights
    pub fn flash(time: Option<Duration>) -> Self {
        Self {
            input: AiInputType::Flash(1),
            time,
        }
    }

    /// Clutch: 0 to 65535
    pub fn clutch(value: u16, time: Option<Duration>) -> Self {
        Self {
            input: AiInputType::Clutch(value),
            time,
        }
    }

    /// Handbrake: 0 to 65535
    pub fn handbrake(value: u16, time: Option<Duration>) -> Self {
        Self {
            input: AiInputType::Handbrake(value),
            time,
        }
    }

    /// Indicators cancel
    pub fn indicators_cancel(time: Option<Duration>) -> Self {
        Self {
            input: AiInputType::Indicators(1),
            time,
        }
    }

    /// Indicate left
    pub fn indicators_left(time: Option<Duration>) -> Self {
        Self {
            input: AiInputType::Indicators(2),
            time,
        }
    }

    /// Indicate right
    pub fn indicators_right(time: Option<Duration>) -> Self {
        Self {
            input: AiInputType::Indicators(3),
            time,
        }
    }

    /// Hazards
    pub fn hazards(time: Option<Duration>) -> Self {
        Self {
            input: AiInputType::Indicators(4),
            time,
        }
    }

    /// Gear: 0 to 254, or 255 for sequential
    pub fn gear(value: u8, time: Option<Duration>) -> Self {
        Self {
            input: AiInputType::Gear(value as u16),
            time,
        }
    }

    /// Look ahead
    pub fn look_ahead(time: Option<Duration>) -> Self {
        Self {
            input: AiInputType::Look(0),
            time,
        }
    }

    /// Look left
    pub fn look_left(time: Option<Duration>) -> Self {
        Self {
            input: AiInputType::Look(4),
            time,
        }
    }

    /// Look left+
    pub fn look_left_plus(time: Option<Duration>) -> Self {
        Self {
            input: AiInputType::Look(5),
            time,
        }
    }

    /// Look right
    pub fn look_right(time: Option<Duration>) -> Self {
        Self {
            input: AiInputType::Look(6),
            time,
        }
    }

    /// Look right+
    pub fn look_right_plus(time: Option<Duration>) -> Self {
        Self {
            input: AiInputType::Look(7),
            time,
        }
    }

    /// Toggle pitspeed limiter
    pub fn pitspeed_toggle() -> Self {
        Self {
            input: AiInputType::Pitspeed(AiInputToggle::Toggle),
            time: None,
        }
    }

    /// Enable pitspeed limiter
    pub fn pitspeed_on() -> Self {
        Self {
            input: AiInputType::Pitspeed(AiInputToggle::On),
            time: None,
        }
    }

    /// Disable pitspeed limiter
    pub fn pitspeed_off() -> Self {
        Self {
            input: AiInputType::Pitspeed(AiInputToggle::Off),
            time: None,
        }
    }

    /// Toggle traction_control limiter
    pub fn traction_control_toggle() -> Self {
        Self {
            input: AiInputType::TcDisable(AiInputToggle::Toggle),
            time: None,
        }
    }

    /// Enable traction_control limiter
    pub fn traction_control_on() -> Self {
        Self {
            input: AiInputType::TcDisable(AiInputToggle::Off),
            time: None,
        }
    }

    /// Disable traction_control limiter
    pub fn traction_control_off() -> Self {
        Self {
            input: AiInputType::TcDisable(AiInputToggle::On),
            time: None,
        }
    }

    /// Stop all control
    pub fn stop_control() -> Self {
        Self {
            input: AiInputType::StopControl,
            time: None,
        }
    }

    /// Reset all inputs
    pub fn reset_all_inputs() -> Self {
        Self {
            input: AiInputType::ResetAll,
            time: None,
        }
    }

    /// Request LFS sends AI Info once
    pub fn ai_info_once() -> Self {
        Self {
            input: AiInputType::SendAiInfo,
            time: None,
        }
    }

    /// Request LFS sends AI Info at a given interval, or 0 to stop
    pub fn ai_info_repeat(time: Duration) -> Self {
        Self {
            input: AiInputType::RepeatAiInfo,
            time: Some(time),
        }
    }

    /// Set the help flags
    pub fn set_help_flags(flags: AiHelpFlags) -> Self {
        Self {
            input: AiInputType::SetHelpFlags(flags),
            time: None,
        }
    }
}

impl Decode for AiInput {
    fn decode(buf: &mut bytes::Bytes) -> Result<Self, insim_core::DecodeError> {
        let input = u8::decode(buf)?;
        let time = u8::decode(buf)?;
        let val = u16::decode(buf)?;

        let input = match input {
            0 => AiInputType::Msx(val),
            1 => AiInputType::Throttle(val),
            2 => AiInputType::Brake(val),
            3 => AiInputType::Chup(val),
            4 => AiInputType::Chdn(val),
            5 => AiInputType::Ignition(val.into()),
            6 => AiInputType::ExtraLight(val.into()),
            7 => AiInputType::HeadLights(val),
            8 => AiInputType::Siren(val),
            9 => AiInputType::Horn(val),
            10 => AiInputType::Flash(val),
            11 => AiInputType::Clutch(val),
            12 => AiInputType::Handbrake(val),
            13 => AiInputType::Indicators(val),
            14 => AiInputType::Gear(val),
            15 => AiInputType::Look(val),
            16 => AiInputType::Pitspeed(val.into()),
            17 => AiInputType::TcDisable(val.into()),
            18 => AiInputType::FogRear(val.into()),
            19 => AiInputType::FogFront(val.into()),
            240 => AiInputType::SendAiInfo,
            241 => AiInputType::RepeatAiInfo,
            253 => {
                let flags = AiHelpFlags::from_bits_truncate(val);
                AiInputType::SetHelpFlags(flags)
            },
            254 => AiInputType::ResetAll,
            255 => AiInputType::StopControl,
            found => {
                return Err(insim_core::DecodeError::NoVariantMatch {
                    found: found as u64,
                });
            },
        };

        let time = if time == 0 {
            None
        } else {
            Some(Duration::from_millis(time as u64 * 10))
        };

        Ok(Self { input, time })
    }
}

impl Encode for AiInput {
    fn encode(&self, buf: &mut bytes::BytesMut) -> Result<(), insim_core::EncodeError> {
        let (discrim, val): (u8, u16) = match self.input {
            AiInputType::Msx(val) => (0, val),
            AiInputType::Throttle(val) => (1, val),
            AiInputType::Brake(val) => (2, val),
            AiInputType::Chup(val) => (3, val),
            AiInputType::Chdn(val) => (4, val),
            AiInputType::Ignition(val) => (5, val.into()),
            AiInputType::ExtraLight(val) => (6, val.into()),
            AiInputType::HeadLights(val) => (7, val),
            AiInputType::Siren(val) => (8, val),
            AiInputType::Horn(val) => (9, val),
            AiInputType::Flash(val) => (10, val),
            AiInputType::Clutch(val) => (11, val),
            AiInputType::Handbrake(val) => (12, val),
            AiInputType::Indicators(val) => (13, val),
            AiInputType::Gear(val) => (14, val),
            AiInputType::Look(val) => (15, val),
            AiInputType::Pitspeed(val) => (16, val.into()),
            AiInputType::TcDisable(val) => (17, val.into()),
            AiInputType::FogRear(val) => (18, val.into()),
            AiInputType::FogFront(val) => (19, val.into()),
            AiInputType::SendAiInfo => (240, 0),
            AiInputType::RepeatAiInfo => (241, 0),
            AiInputType::SetHelpFlags(val) => (253, val.bits()),
            AiInputType::ResetAll => (254, 0),
            AiInputType::StopControl => (255, 0),
        };

        discrim.encode(buf)?;

        if let Some(time) = self.time {
            match u8::try_from(time.as_millis() / 10) {
                Ok(time) => time.encode(buf)?,
                Err(_) => return Err(insim_core::EncodeError::TooLarge),
            }
        } else {
            0_u8.encode(buf)?;
        }

        val.encode(buf)?;
        Ok(())
    }
}

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// AI Input Control
pub struct Aic {
    /// Non-zero if the packet is a packet request or a reply to a request
    pub reqi: RequestId,

    /// Set to choose
    pub plid: PlayerId,

    /// Inputs to send, there are helper methods on [AiInput] to assist building these
    pub inputs: Vec<AiInput>,
}

impl_typical_with_request_id!(Aic);

impl Decode for Aic {
    fn decode(buf: &mut bytes::Bytes) -> Result<Self, insim_core::DecodeError> {
        let reqi = RequestId::decode(buf)?;
        let plid = PlayerId::decode(buf)?;
        let mut inputs = Vec::new();
        while buf.has_remaining() {
            inputs.push(AiInput::decode(buf)?);
        }

        Ok(Self { reqi, plid, inputs })
    }
}

impl Encode for Aic {
    fn encode(&self, buf: &mut bytes::BytesMut) -> Result<(), insim_core::EncodeError> {
        self.reqi.encode(buf)?;
        self.plid.encode(buf)?;
        if self.inputs.len() > AIC_MAX_INPUTS {
            return Err(insim_core::EncodeError::TooLarge);
        }
        for i in self.inputs.iter() {
            i.encode(buf)?;
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
