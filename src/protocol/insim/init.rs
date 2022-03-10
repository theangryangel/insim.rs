use crate::packet_flags;
use crate::string::istring;
use deku::prelude::*;
#[cfg(feature = "serde")]
use serde::Serialize;

packet_flags! {
    /// Flags for the [Init] packet flags field.
    #[cfg_attr(feature = "serde", derive(Serialize))]
    pub struct InitFlags: u16 {
        //RES0 => (1 << 0),	// bit  0: spare
        //RES_1 => (1 << 1),	// bit  1: spare
        LOCAL => (1 << 2),	// bit  2: guest or single player
        MSO_COLS => (1 << 3),	// bit  3: keep colours in MSO text
        NLP => (1 << 4),	// bit  4: receive NLP packets
        MCI => (1 << 5),	// bit  5: receive MCI packets
        CON => (1 << 6),	// bit  6: receive CON packets
        OBH => (1 << 7),	// bit  7: receive OBH packets
        HLV => (1 << 8),	// bit  8: receive HLV packets
        AXM_LOAD => (1 << 9),	// bit  9: receive AXM when loading a layout
        AXM_EDIT => (1 << 10),	// bit 10: receive AXM when changing objects
        REQ_JOIN => (1 << 11),	// bit 11: process join requests
    }
}

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[deku(ctx = "_endian: deku::ctx::Endian")]
/// Insim Init, or handshake packet.
/// Required to be sent to the server before any other packets.
pub struct Init {
    /// When set to a non-zero value the server will send a [Version](super::Version) packet in response.
    ///packet in response.
    #[deku(pad_bytes_after = "1")]
    pub reqi: u8,

    // we do not support this feature, using pad_bytes_before
    // on flags to mask it.
    //#[deku(bytes = "2")]
    //pub udpport: u16,
    /// Options for the Insim Connection. See [InitFlags] for more information.
    #[deku(pad_bytes_before = "2")]
    pub flags: InitFlags,

    /// Protocol version of Insim you wish to use.
    pub version: u8,

    /// Messages typed with this prefix will be sent to your InSim program
    /// on the host (in IS_MSO) and not displayed on anyone's screen.
    /// This should be a single ascii character. i.e. b'!'.
    pub prefix: u8,

    /// Time in milliseconds between each [Nlp](super::Nlp) or [Mci](super::Mci) packet when set to a non-zero value and
    /// the relevant flags are set.
    pub interval: u16,

    /// Administrative password.
    #[deku(
        reader = "istring::read(deku::rest, 16)",
        writer = "istring::write(deku::output, &self.password, 16)"
    )]
    pub password: String,

    /// Name of the program.
    #[deku(
        reader = "istring::read(deku::rest, 16)",
        writer = "istring::write(deku::output, &self.name, 16)"
    )]
    pub name: String,
}
