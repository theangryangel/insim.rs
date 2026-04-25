use bitflags::bitflags;
use insim_core::{
    Decode, DecodeContext, Encode, EncodeContext, angvel::AngVel, coordinate::Coordinate,
    heading::Heading, speed::Speed,
};

use crate::identifiers::{PlayerId, RequestId};

/// CompCar direction scale: 32768 units = 180°
pub(super) const COMPCAR_DEGREES_PER_UNIT: f64 = 180.0 / 32768.0;

bitflags! {
    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy, Default)]
    /// Additional state for a car in [Mci].
    pub struct CompCarInfo: u8 {
        /// This car is in the way of a driver who is a lap ahead
        const BLUE = (1 << 0);

        /// This car is slow or stopped and in a dangerous place
        const YELLOW = (1 << 1);

        /// This car is outside the path
        const OOB = (1 << 2);

        /// This car is lagging (missing or delayed position packets)
        const LAG = (1 << 5);

        /// This is the first compcar in this set of MCI packets
        const FIRST = (1 << 6);

        /// This is the last compcar in this set of MCI packets
        const LAST = (1 << 7);
    }
}
impl_bitflags_json_schema!(CompCarInfo);

generate_bitflag_helpers! {
    CompCarInfo,

    pub has_blue_flag => BLUE,
    pub has_yellow_flag => YELLOW,
    pub is_lagging => LAG,
    pub is_first => FIRST,
    pub is_last => LAST,
    pub out_of_bounds => OOB
}

impl_bitflags_from_to_bytes!(CompCarInfo, u8);

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
/// Per-car telemetry entry used by [Mci].
pub struct CompCar {
    /// Index of the last node the player passed.
    pub node: u16,

    /// Current lap number.
    pub lap: u16,

    /// Player identifier.
    pub plid: PlayerId,

    /// Race position.
    pub position: u8,

    /// Additional state flags.
    pub info: CompCarInfo,

    /// World position in meters.
    pub xyz: Coordinate,

    /// Speed.
    pub speed: Speed,

    /// Direction of motion (heading of velocity).
    pub direction: Heading,

    /// Car facing direction.
    pub heading: Heading,

    /// Angular velocity of the car.
    pub angvel: AngVel,
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
    fn decode(ctx: &mut DecodeContext) -> Result<Self, insim_core::DecodeError> {
        let node = ctx.decode::<u16>("node")?;
        let lap = ctx.decode::<u16>("lap")?;
        let plid = ctx.decode::<PlayerId>("plid")?;
        let position = ctx.decode::<u8>("position")?;
        let info = ctx.decode::<CompCarInfo>("info")?;
        ctx.pad("sp3", 1)?;

        let xyz = ctx.decode::<Coordinate>("xyz")?;

        let speed = (ctx.decode::<u16>("speed")? as f32) / 327.68;
        let speed = Speed::from_meters_per_sec(speed);

        let direction_raw = ctx.decode::<u16>("direction")?;
        let direction = Heading::from_degrees((direction_raw as f64) * COMPCAR_DEGREES_PER_UNIT);

        let heading_raw = ctx.decode::<u16>("heading")?;
        let heading = Heading::from_degrees((heading_raw as f64) * COMPCAR_DEGREES_PER_UNIT);

        let angvel = AngVel::from_wire_i16(ctx.decode::<i16>("angvel")?);
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
    fn encode(&self, ctx: &mut EncodeContext) -> Result<(), insim_core::EncodeError> {
        ctx.encode("node", &self.node)?;
        ctx.encode("lap", &self.lap)?;
        ctx.encode("plid", &self.plid)?;
        ctx.encode("position", &self.position)?;
        ctx.encode("info", &self.info)?;
        ctx.pad("sp3", 1)?;
        ctx.encode("xyz", &self.xyz)?;
        let speed = (self.speed.to_meters_per_sec() * 327.68) as u16;
        ctx.encode("speed", &speed)?;

        let direction_units = (self.direction.to_degrees() / COMPCAR_DEGREES_PER_UNIT)
            .round()
            .clamp(0.0, 65535.0) as u16;
        ctx.encode("direction", &direction_units)?;

        let heading_units = (self.heading.to_degrees() / COMPCAR_DEGREES_PER_UNIT)
            .round()
            .clamp(0.0, 65535.0) as u16;
        ctx.encode("heading", &heading_units)?;

        ctx.encode("angvel", &self.angvel.to_wire_i16())?;
        Ok(())
    }
}

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
/// Multi-car telemetry updates.
///
/// - Contains position, speed, and heading for a subset of players.
/// - Large grids are sent across multiple packets.
pub struct Mci {
    /// Request identifier echoed by replies.
    pub reqi: RequestId,

    /// Telemetry entries for a subset of players.
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
    fn decode(ctx: &mut DecodeContext) -> Result<Self, insim_core::DecodeError> {
        let reqi = ctx.decode::<RequestId>("reqi")?;
        let mut numc = ctx.decode::<u8>("numc")?;
        let mut info = Vec::with_capacity(numc as usize);
        while numc > 0 {
            info.push(ctx.decode::<CompCar>("info")?);
            numc -= 1;
        }
        Ok(Self { reqi, info })
    }
}

impl Encode for Mci {
    fn encode(&self, ctx: &mut EncodeContext) -> Result<(), insim_core::EncodeError> {
        ctx.encode("reqi", &self.reqi)?;
        let numc = self.info.len();
        if numc > 255 {
            return Err(insim_core::EncodeErrorKind::OutOfRange {
                min: 0,
                max: 255,
                found: numc,
            }
            .context("Mci::numc too many infos"));
        }
        ctx.encode("numc", &(numc as u8))?;
        for i in self.info.iter() {
            ctx.encode("info", i)?;
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
