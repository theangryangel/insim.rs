use std::time::Duration;

use insim_core::{
    binrw::{self, binrw},
    duration::{binrw_parse_duration, binrw_write_duration},
};

use crate::identifiers::{PlayerId, RequestId};

use super::CompCarInfo;

#[binrw]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// Used within [Con] packet to give a break down of information about the Contact between the two
/// players.
pub struct ConInfo {
    /// Unique player id
    pub plid: PlayerId,

    #[brw(pad_after = 1)]
    /// Additional information
    pub info: CompCarInfo,

    /// Front wheel steer in degrees (right positive)
    pub steer: u8,

    /// High 4 bits: throttle / low 4 bits: brake (0 to 15)
    pub thrbrk: u8, // TODO split into a pair of u4

    /// high 4 bits: clutch      / low 4 bits: handbrake (0 to 15)
    pub cluhan: u8, // TODO split into a pair of u4

    /// high 4 bits: gear (15=R) / low 4 bits: spare
    pub gearsp: u8, // TODO split into a pair of u4

    /// Speed in m/s
    pub speed: u8,

    /// Car's motion if Speed > 0: 0 = world y direction, 128 = 180 deg
    pub direction: u8,

    /// direction of forward axis: 0 = world y direction, 128 = 180 deg
    pub heading: u8,

    /// m/s^2 longitudinal acceleration (forward positive)
    pub accelf: u8,

    /// m/s^2 lateral acceleration (right positive)
    pub acelr: u8,

    /// position (1 metre = 16)
    pub x: i16,

    /// position (1 metre = 16)
    pub y: i16,
}

#[binrw]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// Contact between 2 vehicles
pub struct Con {
    #[brw(pad_after = 1)]
    /// Non-zero if the packet is a packet request or a reply to a request
    pub reqi: RequestId,

    /// high 4 bits: reserved / low 12 bits: closing speed (10 = 1 m/s)
    pub spclose: u16, // TODO strongly type

    #[br(parse_with = binrw_parse_duration::<u16, 10, _>)]
    #[bw(write_with = binrw_write_duration::<u16, 10, _>)]
    /// Time since last reset. Warning this is looping.
    pub time: Duration,

    /// Contact information for vehicle A
    pub a: ConInfo,

    /// Contact information for vehicle B
    pub b: ConInfo,
}
