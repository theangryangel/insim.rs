//! Dashlights

bitflags::bitflags! {
    /// Dashboard indicator lights.
    ///
    /// - Bitflags can be combined and queried with `.contains`.
    /// - Typically reported in telemetry packets.
    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy, Default)]
    pub struct DashLights: u32 {
        /// Shift light
        const SHIFT = 1;
        /// Fullbeam
        const FULLBEAM = (1 << 1);
        /// Handbrake
        const HANDBRAKE = (1 << 2);
        /// Pitspeed limiter
        const PITSPEED = (1 << 3);
        /// Traction control
        const TC = (1 << 4);
        /// Left turn
        const SIGNAL_L = (1 << 5);
        /// Right turn
        const SIGNAL_R = (1 << 6);
        /// Hazards
        const SIGNAL_ANY = (1 << 7);
        /// Oil pressure warning
        const OILWARN = (1 << 8);
        /// Battery warning
        const BATTERY = (1 << 9);
        /// ABS
        const ABS = (1 << 10);
        /// Engine damage
        const ENGINE = (1 << 11);
        /// Rear fog lights
        const FOG_REAR = (1 << 12);
        /// Front fog lights
        const FOG_FRONT = (1 << 13);
        /// Dipped headlights
        const DIPPED = (1 << 14);
        /// Low fuel warning
        const FUELWARN = (1 << 15);
        /// Sidelights
        const SIDELIGHTS = (1 << 16);
        /// Neutral
        const NEUTRAL = (1 << 17);
        /// Severe engine damage
        const ENGINE_SEVERE = (1 << 28);
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for DashLights {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        crate::bitflags_serde::serialize(self, serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for DashLights {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        crate::bitflags_serde::deserialize(deserializer)
    }
}

#[cfg(feature = "schemars")]
impl schemars::JsonSchema for DashLights {
    fn schema_name() -> ::std::borrow::Cow<'static, str> {
        "DashLights".into()
    }

    fn json_schema(_gen: &mut schemars::SchemaGenerator) -> schemars::Schema {
        let names: Vec<String> = <DashLights as ::bitflags::Flags>::FLAGS
            .iter()
            .map(|f: &::bitflags::Flag<DashLights>| f.name().to_owned())
            .collect();
        schemars::json_schema!({
            "type": "array",
            "items": { "type": "string", "enum": names },
            "uniqueItems": true
        })
    }
}

impl crate::Encode for DashLights {
    fn encode(&self, ctx: &mut crate::EncodeContext) -> Result<(), crate::EncodeError> {
        ctx.encode("bits", &self.bits())
    }
}

impl crate::Decode for DashLights {
    fn decode(ctx: &mut crate::DecodeContext) -> Result<Self, crate::DecodeError> {
        ctx.decode::<u32>("bits").map(Self::from_bits_truncate)
    }
}
