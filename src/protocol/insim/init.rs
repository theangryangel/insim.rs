use crate::packet_flags;
use crate::string::IString;
use deku::prelude::*;
use serde::Serialize;

packet_flags! {
    #[derive(Serialize)]
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

#[derive(Debug, PartialEq, DekuRead, DekuWrite, Clone, Serialize)]
#[deku(ctx = "_endian: deku::ctx::Endian")]
pub struct Init {
    #[deku(bytes = "1", pad_bytes_after = "1")]
    pub reqi: u8,

    // we do not support this feature, using pad_bytes_before
    // on flags to mask it.
    //#[deku(bytes = "2")]
    //pub udpport: u16,
    #[deku(bytes = "2", pad_bytes_before = "2")]
    pub flags: InitFlags,

    #[deku(bytes = "1")]
    pub version: u8,

    #[deku(bytes = "1")]
    pub prefix: u8,

    #[deku(bytes = "2")]
    pub interval: u16,

    #[deku(bytes = "16")]
    pub password: IString,

    #[deku(bytes = "16")]
    pub name: IString,
}
