use bytes::BytesMut;
use digest::Digest;
use insim_core::{Encode, EncodeContext, vehicle::Vehicle};

use super::{Passengers, TyreCompound};
use crate::identifiers::{PlayerId, RequestId};

bitflags::bitflags! {
    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy, Default)]
    /// Setup option flags.
    pub struct SetupFlags: u8 {
        /// Patch X setup format (set=1) vs. older format (set=0).
        const PATCH_X = (1 << 7);
        /// ABS enabled.
        const ABS = (1 << 2);
        /// Traction Control enabled.
        const TC = (1 << 1);
        /// Asymmetrical setup.
        const ASYMMETRICAL = (1 << 0);
    }
}
impl_bitflags_json_schema!(SetupFlags, "SetupFlag");
impl_bitflags_from_to_bytes!(SetupFlags, u8);

#[derive(Debug, Default, Clone, PartialEq, insim_core::Decode, insim_core::Encode)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[repr(u8)]
#[non_exhaustive]
/// Tyre brand.
pub enum TyreBrand {
    /// Cromo plain
    #[default]
    CromoPlain = 0,
    /// Cromo
    Cromo = 1,
    /// Torro
    Torro = 2,
    /// Michelin
    Michelin = 3,
    /// Evostar
    Evostar = 4,
}

#[derive(Debug, Default, Clone, PartialEq, insim_core::Decode, insim_core::Encode)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[repr(u8)]
#[non_exhaustive]
/// Centre differential type.
pub enum CentreDiffType {
    /// Open
    #[default]
    Open = 0,
    /// Viscous
    Viscous = 1,
}

#[derive(Debug, Default, Clone, PartialEq, insim_core::Decode, insim_core::Encode)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[repr(u8)]
#[non_exhaustive]
/// Differential type.
pub enum DiffType {
    /// Open
    #[default]
    Open = 0,
    /// Locked
    Locked = 1,
    /// Viscous
    Viscous = 2,
    /// Clutch pack
    ClutchPack = 3,
}

#[derive(Debug, Default, Clone, PartialEq, insim_core::Decode, insim_core::Encode)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
/// Vehicle setup.
///
/// Wire format matches the SET file minus the first 12 header bytes, with one
/// difference: gear ratios are ordered gears 1-7 then FDR (in the file it is 7th gear then
/// FDR then gears 1-6).
///
/// # Note
///
/// The SET file format is reverse-engineered rather than formally documented
/// As a result, some fields are left as raw integer types rather than strongly-typed
/// enums or newtypes, to avoid silent breakage if the format is updated or our understanding
/// turns out to be incomplete.
pub struct VehicleSetup {
    /// Setup option flags (ABS, TC, Asymmetrical, Patch-X format).
    pub flags: SetupFlags,

    /// Unknown byte.
    pub unknown: u8,

    /// Handicap mass position.
    pub handicap_mass_position: u8,

    /// Tyre brand.
    pub tyre_brand: TyreBrand,

    /// Brake strength (Nm).
    pub brake_strength: f32,

    /// Rear wing angle.
    pub rear_wing_angle: u8,

    /// Front wing angle.
    pub front_wing_angle: u8,

    /// Voluntary handicap mass.
    pub voluntary_handicap_mass: u8,

    /// Voluntary intake restriction.
    pub voluntary_intake_restriction: u8,

    /// Maximum steering lock.
    pub max_steering_lock: u8,

    /// Parallel steering.
    pub parallel_steering: u8,

    /// Brake balance.
    pub brake_balance: u8,

    /// Engine brake reduction.
    pub engine_brake_reduction: u8,

    /// Centre differential type.
    pub centre_diff_type: CentreDiffType,

    /// Centre differential viscous torque.
    #[insim(pad_after = 1)]
    pub centre_diff_viscous_torque: u8,

    /// Centre differential torque split.
    pub centre_diff_torque_split: u8,

    /// Gear ratios for gears 1-7 followed by the final drive ratio (0-65534 maps to 0.5-7.5).
    ///
    /// Index 0 = 1st gear, index 6 = 7th gear, index 7 = final drive.
    pub gear_ratios: [u16; 8],

    /// Passenger layout.
    pub passengers: Passengers,

    /// Car configuration (e.g. roof on LX4/6 and UF1).
    pub car_config: u8,

    /// Traction control slip (divide by ten for the actual value).
    pub tc_slip: u8,

    /// Traction control engagement speed.
    pub tc_engage_speed: u8,

    /// Rear ride height in mm (not spring motion range).
    pub rear_ride_height: f32,

    /// Rear spring stiffness (N/mm).
    pub rear_spring_stiffness: f32,

    /// Rear compression/bump damping (N/mm).
    pub rear_bump_damping: f32,

    /// Rear rebound damping (N/mm).
    pub rear_rebound_damping: f32,

    /// Rear anti-roll bar stiffness (N/mm).
    pub rear_anti_roll_bar: f32,

    /// Handbrake strength (N.m).
    pub handbrake_strength: f32,

    /// Rear toe (0=-0.9deg, 9=0deg, 18=0.9deg).
    pub rear_toe: u8,

    /// Rear caster (always zero).
    pub rear_caster: u8,

    /// Rear tyre type (R1 through Knobbly).
    pub rear_tyre_type: TyreCompound,

    /// Rear tyre warmer temperature.
    pub rear_tyre_warmer_temp: u8,

    /// Rear left camber adjust (45=0.0deg, 0=-4.5deg, 90=4.5deg).
    pub rear_left_camber: u8,

    /// Rear right camber adjust (45=0.0deg, 0=-4.5deg, 90=4.5deg).
    pub rear_right_camber: u8,

    /// Rear tyre size (alternate GTR configuration).
    pub rear_tyre_size: u8,

    /// Rear diff clutch pack pre-load (multiply by ten).
    pub rear_diff_clutch_preload: u8,

    /// Rear differential type.
    pub rear_diff_type: DiffType,

    /// Rear viscous torque.
    pub rear_viscous_torque: u8,

    /// Rear power locking.
    pub rear_power_locking: u8,

    /// Rear coast locking.
    pub rear_coast_locking: u8,

    /// Rear left tyre pressure (kPa).
    pub rear_left_tyre_pressure: u16,

    /// Rear right tyre pressure (kPa).
    pub rear_right_tyre_pressure: u16,

    /// Front ride height in mm (not spring motion range).
    pub front_ride_height: f32,

    /// Front spring stiffness (N/mm).
    pub front_spring_stiffness: f32,

    /// Front bump/compression damping (N/mm).
    pub front_bump_damping: f32,

    /// Front rebound damping (N/mm).
    pub front_rebound_damping: f32,

    /// Front anti-roll bar stiffness (N/mm).
    #[insim(pad_after = 4)]
    pub front_anti_roll_bar: f32,

    /// Front toe in (0=-0.9deg, 9=0deg, 18=0.9deg).
    pub front_toe: u8,

    /// Front caster (divide by ten).
    pub front_caster: u8,

    /// Front tyre type (R1 through Knobbly).
    pub front_tyre_type: TyreCompound,

    /// Front tyre warmer temperature.
    pub front_tyre_warmer_temp: u8,

    /// Front left camber adjust (45=0.0deg, 0=-4.5deg, 90=4.5deg).
    pub front_left_camber: u8,

    /// Front right camber adjust (45=0.0deg, 0=-4.5deg, 90=4.5deg).
    pub front_right_camber: u8,

    /// Front tyre size (alternate GTR configuration).
    pub front_tyre_size: u8,

    /// Front diff clutch pack pre-load (multiply by ten).
    pub front_diff_clutch_preload: u8,

    /// Front differential type.
    pub front_diff_type: DiffType,

    /// Front viscous torque.
    pub front_viscous_torque: u8,

    /// Front power locking.
    pub front_power_locking: u8,

    /// Front coast locking.
    pub front_coast_locking: u8,

    /// Front left tyre pressure (kPa).
    pub front_left_tyre_pressure: u16,

    /// Front right tyre pressure (kPa).
    pub front_right_tyre_pressure: u16,
}

impl VehicleSetup {
    const SET_PAYLOAD_LEN: usize = 120;

    /// Digest of this setup encoded in SET file payload order.
    ///
    /// The result is equivalent to hashing bytes 12 onwards of the corresponding SET file on
    /// disk - the 12-byte file header is skipped, and the remaining 120 bytes are hashed
    /// directly. To pre-compute an approved digest from the command line:
    ///
    /// ```text
    /// # SHA-256
    /// tail -c +13 approved.set | sha256sum
    ///
    /// # SHA-512
    /// tail -c +13 approved.set | sha512sum
    /// ```
    ///
    /// Or in Rust:
    ///
    /// ```rust,ignore
    /// let bytes = std::fs::read(path)?;
    /// let hash = sha2::Sha256::digest(&bytes[12..]);
    /// ```
    pub fn digest<D: Digest>(&self) -> digest::Output<D> {
        let mut buf = BytesMut::with_capacity(Self::SET_PAYLOAD_LEN);
        let mut ctx = EncodeContext::new(&mut buf);
        self.encode(&mut ctx)
            .expect("VehicleSetup encode is infallible");

        // Reorder gears from packet order [g1..g6, g7, FDR] to file order [g7, FDR, g1..g6].
        let mut payload = [0u8; 120];
        payload.copy_from_slice(&buf);
        payload[20..22].copy_from_slice(&buf[32..34]); // g7
        payload[22..24].copy_from_slice(&buf[34..36]); // FDR
        payload[24..36].copy_from_slice(&buf[20..32]); // g1..g6

        D::digest(payload)
    }
}

#[derive(Debug, Clone, Default, insim_core::Decode, insim_core::Encode)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
/// Player sent setup to host
///
/// - Sent when SET is enabled in [IsiFlags](crate::insim::IsiFlags).
pub struct Set {
    /// Request identifier echoed by replies.
    pub reqi: RequestId,

    /// Player that left the race.
    pub plid: PlayerId,

    /// vehicle
    #[insim(pad_after = 4)]
    pub cname: Vehicle,

    /// Fuel Load at start
    #[insim(pad_after = 3)]
    pub fuelload: u8,

    /// Vehicle setup.
    pub setup: VehicleSetup,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_vehicle_setup_encoded_size() {
        let mut buf = BytesMut::new();
        let mut ctx = EncodeContext::new(&mut buf);
        VehicleSetup::default().encode(&mut ctx).unwrap();
        assert_eq!(buf.len(), 120);
    }

    #[test]
    #[allow(unused_imports)]
    fn test_digest_matches_set_file_payload() {
        use digest::Digest as _;
        let setup = VehicleSetup::default();

        // Encode in packet order then apply file gear order [g7, FDR, g1..g6].
        let mut buf = BytesMut::with_capacity(120);
        let mut ctx = EncodeContext::new(&mut buf);
        setup.encode(&mut ctx).unwrap();
        let packet_payload = buf.freeze();

        let mut file_payload = [0u8; 120];
        file_payload.copy_from_slice(&packet_payload);
        file_payload[20..22].copy_from_slice(&packet_payload[32..34]); // g7
        file_payload[22..24].copy_from_slice(&packet_payload[34..36]); // FDR
        file_payload[24..36].copy_from_slice(&packet_payload[20..32]); // g1..g6

        assert_eq!(
            setup.digest::<sha2::Sha512>(),
            sha2::Sha512::digest(file_payload)
        );
    }

    #[test]
    fn test_set() {
        assert_from_to_bytes!(
            Set,
            [
                1,  // reqi
                12, // plid
                b'X', b'F', b'G', 0, // cname = XFG
                0, 0, 0, 0,  // pad_after = 4
                10, // fuelload
                0, 0, 0, // pad_after = 3
                // setup[0]: flags = TC | ASYMMETRICAL = 0x03
                0x03, // setup[1]: unknown
                0,    // setup[2]: handicap_mass_position
                0,    // setup[3]: tyre_brand = Michelin (3)
                3,    // setup[4..8]: brake_strength = 1000.0f32 little-endian
                0x00, 0x00, 0x7A, 0x44, // setup[8]: rear_wing_angle = 10
                10,
                // setup[9..20]: front_wing through centre_diff_torque_split (includes 1 pad at [18])
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                // setup[20..36]: gear_ratios [u16; 8] all zero
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                // setup[36..66]: passengers through rear_caster (all zero)
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, // setup[66]: rear_tyre_type = R3 (2)
                2, // setup[67..72]: warmer, cambers, tyre_size, clutch_preload
                0, 0, 0, 0, 0, // setup[72]: rear_diff_type = Locked (1)
                1,
                // setup[73..106]: viscous through front_caster (all zero, includes 4 pad bytes at [100..104])
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, // setup[106]: front_tyre_type = RoadNormal (5)
                5, // setup[107..112]: warmer, cambers, tyre_size, clutch_preload
                0, 0, 0, 0, 0, // setup[112]: front_diff_type = Viscous (2)
                2, // setup[113..120]: viscous, power_lock, coast_lock, pressures
                0, 0, 0, 0, 0, 0, 0,
            ],
            |parsed: Set| {
                assert_eq!(parsed.reqi, RequestId(1));
                assert_eq!(parsed.plid, PlayerId(12));
                assert_eq!(parsed.cname, Vehicle::Xfg);
                assert_eq!(parsed.fuelload, 10);
                assert_eq!(
                    parsed.setup.flags,
                    SetupFlags::TC | SetupFlags::ASYMMETRICAL
                );
                assert!(matches!(parsed.setup.tyre_brand, TyreBrand::Michelin));
                assert_eq!(parsed.setup.brake_strength, 1000.0_f32);
                assert_eq!(parsed.setup.rear_wing_angle, 10);
                assert!(matches!(parsed.setup.rear_tyre_type, TyreCompound::R3));
                assert!(matches!(parsed.setup.rear_diff_type, DiffType::Locked));
                assert!(matches!(
                    parsed.setup.front_tyre_type,
                    TyreCompound::RoadNormal
                ));
                assert!(matches!(parsed.setup.front_diff_type, DiffType::Viscous));
            }
        );
    }
}
