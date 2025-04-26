use std::{ops::Deref, time::Duration};

use bitflags::bitflags;
use insim_core::ReadWriteBuf;

use super::{PlcAllowedCarsSet, VtnAction};
use crate::{
    identifiers::{PlayerId, RequestId},
    Packet, WithRequestId,
};

bitflags! {
    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy, Default)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize))]
    /// Local Car Switches Flags. You probably want to use [LclFlags] instead. This is deprecated,
    /// but kept for backwards compatibility.
    /// Configure and control lights on a vehicle using a [Small] packet.
    pub struct LcsFlags: u32 {
        /// Set indicators/turn signals. You probably want to look at the shortcut options,
        /// prefixed with `SIGNAL_`.
        const SET_SIGNALS = (1 << 0);

        /// Set flash. You probably want to look at the helper options prefixed with `FLASH_`
        const SET_FLASH = (1 << 1);

        /// Set headlights. You probably want to look at the helper options prefixed with
        /// `HEADLIGHTS_`
        const SET_HEADLIGHTS = (1 << 2);

        /// Set horn. You probably want to look at the helper options prefixed with `HORN_`.
        const SET_HORN = (1 << 3);

        /// Set siren. You probably want to look at the helper options prefixed with `SIREN_`.
        const SET_SIREN = (1 << 4);

        /// Shortcut to turn off all signals/indicators
        const SIGNAL_OFF = Self::SET_SIGNALS.bits();
        /// Enable left indicator/signal
        const SIGNAL_LEFT = Self::SET_SIGNALS.bits() | (1 << 8);
        /// Enable right indicator/signal
        const SIGNAL_RIGHT = Self::SET_SIGNALS.bits() | (2 << 8);
        /// Enable both left and right indicators/signals, aka hazard lights
        const SIGNAL_HAZARD = Self::SET_SIGNALS.bits() | (3 << 8);

        /// Disable Flash
        const FLASH_OFF = Self::SET_FLASH.bits();
        /// Enable Flash
        const FLASH_ON = Self::SET_FLASH.bits() | (1 << 10);

        /// Disable headlights
        const HEADLIGHTS_OFF = Self::SET_HEADLIGHTS.bits();
        /// Enable headlights
        const HEADLIGHTS_ON = Self::SET_HEADLIGHTS.bits() | (1 << 11);

        /// Disable horn
        const HORN_OFF = Self::SET_HORN.bits();
        /// Use "horn 1"
        const HORN_1 = Self::SET_HORN.bits() | (1 << 16);
        /// Use "horn 2"
        const HORN_2 = Self::SET_HORN.bits() | (2 << 16);
        /// Use "horn 3"
        const HORN_3 = Self::SET_HORN.bits() | (3 << 16);
        /// Use "horn 4"
        const HORN_4 = Self::SET_HORN.bits() | (4 << 16);
        /// Use "horn 5"
        const HORN_5 = Self::SET_HORN.bits() | (5 << 16);

        /// Disable siren
        const SIREN_OFF = Self::SET_SIREN.bits();
        /// Use fast siren
        const SIREN_FAST = Self::SET_SIREN.bits() | (1 << 20);
        /// Use slow siren
        const SIREN_SLOW = Self::SET_SIREN.bits() | (2 << 20);
    }
}

bitflags! {
    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy, Default)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize))]
    /// Local Car Lights Flags.
    /// Configure and control lights on a vehicle using a [Small] packet.
    pub struct LclFlags: u32 {
        /// Set signals. Take a look at the helper options prefixed `SIGNAL_`.
        const SET_SIGNALS = (1 << 0);
        /// Set lights. Take a look at the helper options prefixed `LIGHT_`.
        const SET_LIGHTS = (1 << 2);
        /// Set rear fogs.. Take a look at the helper options prefixed `FOG_FRONT_`.
        const SET_FOG_REAR = (1 << 4);
        /// Set front fogs.. Take a look at the helper options prefixed `FOG_REAR_`.
        const SET_FOG_FRONT = (1 << 5);
        /// Set extra lights.. Take a look at the helper options prefixed `EXTRA_`.
        const SET_EXTRA = (1 << 6);

        /// Disable all signals/indicators
        const SIGNAL_OFF = Self::SET_SIGNALS.bits();
        /// Left signal/indicator
        const SIGNAL_LEFT = Self::SET_SIGNALS.bits() | (1 << 16);
        /// Right signal/indicator
        const SIGNAL_RIGHT = Self::SET_SIGNALS.bits() | (2 << 16);
        /// Hazards, aka both left and right signal/indicator
        const SIGNAL_HAZARD = Self::SET_SIGNALS.bits() | (3 << 16);

        /// Disable all lights
        const LIGHT_OFF = Self::SET_LIGHTS.bits();
        /// Enable side lights
        const LIGHT_SIDE = Self::SET_LIGHTS.bits() | (1 << 18);
        /// Enable low beam lights
        const LIGHT_LOW = Self::SET_LIGHTS.bits() | (2 << 18);
        /// Enable high beam lights
        const LIGHT_HIGH = Self::SET_LIGHTS.bits() | (3 << 18);

        /// Disable rear fogs
        const FOG_REAR_OFF = Self::SET_FOG_REAR.bits();
        /// Enable rear fogs
        const FOG_REAR = Self::SET_FOG_REAR.bits() | (1 << 20);

        /// Disable front fogs
        const FOG_FRONT_OFF = Self::SET_FOG_FRONT.bits();
        /// Disable front fogs
        const FOG_FRONT = Self::SET_FOG_FRONT.bits() | (1 << 21);

        /// Disable all "extra" lights
        const EXTRA_OFF = Self::SET_EXTRA.bits();
        /// Enable all "extra" lights
        const EXTRA = Self::SET_EXTRA.bits() | (1 << 2);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[non_exhaustive]
/// [Small] packet subtype.
pub enum SmallType {
    /// Nothing!
    None,

    /// Request LFS to start sending positions
    Ssp(Duration),

    /// Request LFS to start sending gauges
    Ssg(Duration),

    /// Vote action
    Vta(VtnAction),

    /// Time stop
    Tms(bool),

    /// Time step
    Stp(Duration),

    /// Race time packet (reply to Gth)
    Rtp(Duration),

    /// Set node lap interval
    Nli(Duration),

    /// Set or get allowed cars (Tiny, type = Alc)
    Alc(PlcAllowedCarsSet),

    /// Set local car switches
    Lcs(LcsFlags),

    /// Set local vehicle lights
    Lcl(LclFlags),

    /// Get local AI information
    Aii(PlayerId),
}

impl Default for SmallType {
    fn default() -> Self {
        Self::None
    }
}

impl From<SmallType> for Packet {
    fn from(value: SmallType) -> Self {
        Self::Small(Small {
            subt: value,
            ..Default::default()
        })
    }
}

impl From<LclFlags> for Packet {
    fn from(value: LclFlags) -> Self {
        Self::Small(Small {
            subt: SmallType::Lcl(value),
            ..Default::default()
        })
    }
}

impl WithRequestId for LclFlags {
    fn with_request_id<R: Into<crate::identifiers::RequestId>>(
        self,
        reqi: R,
    ) -> impl Into<crate::Packet> + std::fmt::Debug {
        Small {
            reqi: reqi.into(),
            subt: SmallType::Lcl(self),
        }
    }
}

impl From<LcsFlags> for Packet {
    fn from(value: LcsFlags) -> Self {
        Self::Small(Small {
            subt: SmallType::Lcs(value),
            ..Default::default()
        })
    }
}

impl WithRequestId for LcsFlags {
    fn with_request_id<R: Into<crate::identifiers::RequestId>>(
        self,
        reqi: R,
    ) -> impl Into<crate::Packet> + std::fmt::Debug {
        Small {
            reqi: reqi.into(),
            subt: SmallType::Lcs(self),
        }
    }
}

impl From<PlcAllowedCarsSet> for Packet {
    fn from(value: PlcAllowedCarsSet) -> Self {
        Self::Small(Small {
            subt: SmallType::Alc(value),
            ..Default::default()
        })
    }
}

impl WithRequestId for PlcAllowedCarsSet {
    fn with_request_id<R: Into<crate::identifiers::RequestId>>(
        self,
        reqi: R,
    ) -> impl Into<crate::Packet> + std::fmt::Debug {
        Small {
            reqi: reqi.into(),
            subt: SmallType::Alc(self),
        }
    }
}

impl WithRequestId for SmallType {
    fn with_request_id<R: Into<crate::identifiers::RequestId>>(
        self,
        reqi: R,
    ) -> impl Into<crate::Packet> + std::fmt::Debug {
        Small {
            reqi: reqi.into(),
            subt: self,
        }
    }
}

impl ReadWriteBuf for SmallType {
    fn read_buf(buf: &mut bytes::Bytes) -> Result<Self, insim_core::Error> {
        let discrim = u8::read_buf(buf)?;
        let uval = u32::read_buf(buf)?;
        let res = match discrim {
            0 => Self::None,
            1 => Self::Ssp(Duration::from_millis(uval as u64 * 10)),
            2 => Self::Ssg(Duration::from_millis(uval as u64 * 10)),
            3 => Self::Vta(uval.into()),
            4 => Self::Tms(uval != 0),
            5 => Self::Stp(Duration::from_millis(uval as u64 * 10)),
            6 => Self::Rtp(Duration::from_millis(uval as u64 * 10)),
            7 => Self::Nli(Duration::from_millis(uval as u64)),
            8 => Self::Alc(PlcAllowedCarsSet::from_bits_truncate(uval)),
            9 => Self::Lcs(LcsFlags::from_bits_truncate(uval)),
            10 => Self::Lcl(LclFlags::from_bits_truncate(uval)),
            11 => Self::Aii(PlayerId(uval as u8)),
            found => {
                return Err(insim_core::Error::NoVariantMatch {
                    found: found as u64,
                })
            },
        };
        Ok(res)
    }

    fn write_buf(&self, buf: &mut bytes::BytesMut) -> Result<(), insim_core::Error> {
        let (discrim, uval) = match self {
            SmallType::None => (0u8, 0u32),
            SmallType::Ssp(uval) => (1u8, uval.as_millis() as u32 / 10),
            SmallType::Ssg(uval) => (2u8, uval.as_millis() as u32 / 10),
            SmallType::Vta(uval) => (3u8, uval.into()),
            SmallType::Tms(uval) => (4u8, *uval as u32),
            SmallType::Stp(uval) => (5u8, uval.as_millis() as u32 / 10),
            SmallType::Rtp(uval) => (6u8, uval.as_millis() as u32 / 10),
            SmallType::Nli(uval) => (7u8, uval.as_millis() as u32),
            SmallType::Alc(uval) => (8u8, uval.bits()),
            SmallType::Lcs(uval) => (9u8, uval.bits()),
            SmallType::Lcl(uval) => (10u8, uval.bits()),
            SmallType::Aii(plid) => (11u8, (*plid.deref() as u32)),
        };

        discrim.write_buf(buf)?;
        uval.write_buf(buf)?;
        Ok(())
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, insim_macros::ReadWriteBuf)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// General purpose Small packet
pub struct Small {
    /// Non-zero if the packet is a packet request or a reply to a request
    pub reqi: RequestId,

    /// Small subtype.
    pub subt: SmallType,
}

impl_typical_with_request_id!(Small);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_small_none() {
        assert_from_to_bytes!(
            Small,
            vec![
                1, // reqi
                0, 0, 0, 0, 0 // subt
            ],
            |parsed: Small| {
                assert_eq!(parsed.subt, SmallType::None);
            }
        );
    }

    #[test]
    fn test_small_ssp() {
        assert_from_to_bytes!(
            Small,
            vec![
                1, // reqi
                1, 100, 0, 0, 0 // subt
            ],
            |parsed: Small| {
                if let SmallType::Ssp(duration) = parsed.subt {
                    assert_eq!(duration, Duration::from_secs(1));
                } else {
                    panic!("Expected SmallType::Ssp, found {:?}", parsed.subt);
                }
            }
        );
    }

    #[test]
    fn test_lcs_flags_signals_hazard() {
        assert_from_to_bytes!(Small, vec![1, 9, 1, 3, 0, 0], |parsed: Small| {
            assert!(matches!(
                parsed,
                Small {
                    reqi: RequestId(1),
                    subt: SmallType::Lcs(LcsFlags::SIGNAL_HAZARD),
                }
            ));
        });
    }

    #[test]
    fn test_lcl_flags_signals_off() {
        assert_from_to_bytes!(Small, vec![1, 10, 1, 0, 0, 0], |parsed: Small| {
            assert!(matches!(
                parsed,
                Small {
                    reqi: RequestId(1),
                    subt: SmallType::Lcl(LclFlags::SIGNAL_OFF),
                }
            ));
        });
    }

    #[test]
    fn test_lcl_flags_signals_hazard() {
        assert_from_to_bytes!(Small, vec![1, 10, 1, 0, 3, 0], |parsed: Small| {
            assert!(matches!(
                parsed,
                Small {
                    reqi: RequestId(1),
                    subt: SmallType::Lcl(LclFlags::SIGNAL_HAZARD),
                }
            ));
        });
    }
}
