use std::time::Duration;

use insim_core::{
    binrw::{self, binrw},
    duration::{binrw_parse_duration, binrw_write_duration},
    identifiers::RequestId,
    string::{binrw_parse_codepage_string, binrw_write_codepage_string},
};

use bitflags::bitflags;

bitflags! {
    /// Flags for the [Init] packet flags field.
    #[binrw]
    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy, Default)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize))]
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
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// Insim Init, or handshake packet.
/// Required to be sent to the server before any other packets.
pub struct Isi {
    /// When set to a non-zero value the server will send a [Version](super::Version) packet in response.
    ///packet in response.
    #[brw(pad_after = 1)]
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
    #[br(parse_with = binrw_parse_duration::<u16, _>)]
    #[bw(write_with = binrw_write_duration::<u16, _>)]
    pub interval: Duration,

    /// Administrative password.
    #[bw(write_with = binrw_write_codepage_string::<16, _>, args(true, 0))]
    #[br(parse_with = binrw_parse_codepage_string::<16, _>, args(true))]
    pub admin: String,

    /// Name of the program.
    #[bw(write_with = binrw_write_codepage_string::<16, _>)]
    #[br(parse_with = binrw_parse_codepage_string::<16, _>)]
    pub iname: String,
}

#[cfg(test)]
mod tests {
    use crate::VERSION;
    use insim_core::{binrw::BinWrite, identifiers::RequestId};
    use std::{io::Cursor, time::Duration};

    use super::Isi;

    #[test]
    fn test_isi() {
        use bytes::BufMut;

        let mut data = Isi::default();
        data.reqi = RequestId(1);
        data.iname = "insim.rs".to_string();
        data.admin = "^JA".to_string();
        data.version = VERSION;
        data.prefix = '!';
        data.interval = Duration::from_secs(1);

        let mut buf = Cursor::new(Vec::new());
        let res = data.write_le(&mut buf);
        assert!(res.is_ok());
        let buf = buf.into_inner();
        assert_eq!(buf.len(), 42); // less magic, less size

        assert_eq!(&buf[0], &0b1); // check that the reqi is 1
        assert_eq!(&buf[6], &VERSION); // check that the version is VERSION
        assert_eq!(&buf[7], &('!' as u8));
        assert_eq!(
            &buf[10..26],
            &[b'^', b'J', b'A', 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
        );

        let mut iname = "insim.rs".to_string().as_bytes().to_owned();
        iname.put_bytes(0, 16 - iname.len());

        assert_eq!(&buf[26..42], &iname,);
    }
}
