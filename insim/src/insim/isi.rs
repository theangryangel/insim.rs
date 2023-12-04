use std::time::Duration;

use insim_core::{
    binrw::{self, binrw},
    duration::{binrw_parse_u16_duration, binrw_write_u16_duration},
    identifiers::RequestId,
    string::{binrw_parse_codepage_string, binrw_write_codepage_string},
};

#[cfg(feature = "serde")]
use serde::Serialize;

use bitflags::bitflags;

bitflags! {
    /// Flags for the [Init] packet flags field.
    #[binrw]
    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy, Default)]
    #[cfg_attr(feature = "serde", derive(Serialize))]
    #[br(map = Self::from_bits_truncate)]
    #[bw(map = |&x: &Self| x.bits())]
    pub struct IsiFlags: u16 {
        //RES0 => (1 << 0),	// bit  0: spare
        //RES_1 => (1 << 1),	// bit  1: spare
         const LOCAL = (1 << 2);	// bit  2: guest or single player
         const MSO_COLS = (1 << 3);	// bit  3: keep colours in MSO text
         const NLP = (1 << 4);	// bit  4: receive NLP packets
         const MCI = (1 << 5);	// bit  5: receive MCI packets
         const CON = (1 << 6);	// bit  6: receive CON packets
         const OBH = (1 << 7);	// bit  7: receive OBH packets
         const HLV = (1 << 8);	// bit  8: receive HLV packets
         const AXM_LOAD = (1 << 9);	// bit  9: receive AXM when loading a layout
         const AXM_EDIT = (1 << 10);	// bit 10: receive AXM when changing objects
         const REQ_JOIN = (1 << 11);	// bit 11: process join requests
    }
}

impl IsiFlags {
    pub fn clear(&mut self) {
        *self.0.bits_mut() = 0;
    }
}

#[binrw]
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
/// Insim Init, or handshake packet.
/// Required to be sent to the server before any other packets.
pub struct Isi {
    /// When set to a non-zero value the server will send a [Version](super::Version) packet in response.
    ///packet in response.
    #[brw(pad_after = 2)]
    pub reqi: RequestId,

    /// UDP Port
    pub udpport: u16,

    /// Options for the Insim Connection. See [IsiFlags] for more information.
    pub flags: IsiFlags,

    pub version: u8,

    /// Messages typed with this prefix will be sent to your InSim program
    /// on the host (in IS_MSO) and not displayed on anyone's screen.
    /// This should be a single ascii character. i.e. '!'.
    #[bw(map = |&x| x as u8)]
    #[br(map = |x: u8| x as char)]
    pub prefix: char,

    /// Time in between each [Nlp](super::Nlp) or [Mci](super::Mci) packet when set to a non-zero value and
    /// the relevant flags are set.
    #[br(parse_with = binrw_parse_u16_duration::<_>)]
    #[bw(write_with = binrw_write_u16_duration::<_>)]
    pub interval: Duration,

    /// Administrative password.
    #[bw(write_with = binrw_write_codepage_string::<16, _>)]
    #[br(parse_with = binrw_parse_codepage_string::<16, _>)]
    pub admin: String,

    /// Name of the program.
    #[bw(write_with = binrw_write_codepage_string::<16, _>)]
    #[br(parse_with = binrw_parse_codepage_string::<16, _>)]
    pub iname: String,
}
