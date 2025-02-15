use std::time::Duration;

use bitflags::bitflags;
use bytes::BufMut;
use insim_core::{
    binrw::{self, binrw},
    duration::{binrw_parse_duration, binrw_write_duration},
    string::{binrw_parse_codepage_string, binrw_write_codepage_string},
    FromToBytes,
};

use crate::{identifiers::RequestId, WithRequestId, VERSION};

bitflags! {
    /// Flags for the [Init] packet flags field.
    #[binrw]
    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy, Default)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize))]
    #[br(map = Self::from_bits_truncate)]
    #[bw(map = |&x: &Self| x.bits())]
    /// Flags for [Isi], used to indicate what behaviours we want to opt into
    pub struct IsiFlags: u16 {
        /// Guest or single player
        const LOCAL = (1 << 2);

        /// Keep colours in MSO text
        const MSO_COLS = (1 << 3);

        /// Receive NLP packets
        const NLP = (1 << 4);

        /// Receive MCI packets
        const MCI = (1 << 5);

        /// Receive CON packets
        const CON = (1 << 6);

        /// Receive OBH packets
        const OBH = (1 << 7);

        /// Receive HLV packets
        const HLV = (1 << 8);

        /// Rceive AXM when loading a layout
        const AXM_LOAD = (1 << 9);

        /// Receive AXM when changing objects
        const AXM_EDIT = (1 << 10);

        /// Process join requests
        const REQ_JOIN = (1 << 11);
    }
}

impl IsiFlags {
    /// Clear all flags
    pub fn clear(&mut self) {
        *self.0.bits_mut() = 0;
    }
}

impl From<IsiFlags> for Isi {
    fn from(value: IsiFlags) -> Self {
        Self {
            flags: value,
            ..Default::default()
        }
    }
}

#[binrw]
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// Insim Init, or handshake packet.
/// Required to be sent to the server before any other packets.
pub struct Isi {
    /// When set to a non-zero value the server will send a [crate::Packet::Ver] packet in response.
    ///packet in response.
    #[brw(pad_after = 1)]
    pub reqi: RequestId,

    /// UDP Port
    pub udpport: u16,

    /// Options for the Insim Connection. See [IsiFlags] for more information.
    pub flags: IsiFlags,

    /// Insim protocol version. Unless you have a good reason you should probably leave this as the
    /// default value of [crate::VERSION].
    pub version: u8,

    /// Messages typed with this prefix will be sent to your InSim program
    /// on the host (in IS_MSO) and not displayed on anyone's screen.
    /// This should be a single ascii character. i.e. '!'.
    #[bw(map = |&x| x as u8)]
    #[br(map = |x: u8| x as char)]
    pub prefix: char,

    /// Time in between each [Nlp](super::Nlp) or [Mci](super::Mci) packet when set to a non-zero value and
    /// the relevant flags are set.
    #[br(parse_with = binrw_parse_duration::<u16, 1, _>)]
    #[bw(write_with = binrw_write_duration::<u16, 1, _>)]
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

impl Isi {
    /// Default application name
    pub const DEFAULT_INAME: &'static str = "insim.rs";
}

impl FromToBytes for Isi {
    fn from_bytes(_buf: &mut bytes::Bytes) -> Result<Self, insim_core::Error> {
        todo!()
    }

    fn to_bytes(&self, buf: &mut bytes::BytesMut) -> Result<(), insim_core::Error> {
        self.reqi.to_bytes(buf)?;
        buf.put_bytes(0, 1);
        self.udpport.to_bytes(buf)?;
        self.flags.bits().to_bytes(buf)?;
        self.version.to_bytes(buf)?;
        self.prefix.to_bytes(buf)?;
        let interval =
            u16::try_from(self.interval.as_millis()).map_err(insim_core::Error::TryFromInt)?;
        interval.to_bytes(buf)?;

        let admin = self.admin.as_bytes();
        if admin.len() >= 16 {
            buf.put(&admin[..16]);
        } else {
            buf.put(admin);
            buf.put_bytes(0, 16 - admin.len());
        }

        let iname = self.iname.as_bytes();
        if iname.len() >= 16 {
            buf.put(&iname[..16]);
        } else {
            buf.put(iname);
            buf.put_bytes(0, 16 - iname.len());
        }

        Ok(())
    }
}

impl Default for Isi {
    fn default() -> Self {
        Self {
            reqi: RequestId(0),
            udpport: 0,
            flags: IsiFlags::default(),
            version: VERSION,
            prefix: 0 as char,
            interval: Duration::default(),
            admin: "".into(),
            iname: Self::DEFAULT_INAME.to_owned(),
        }
    }
}

impl_typical_with_request_id!(Isi);

impl WithRequestId for IsiFlags {
    fn with_request_id<R: Into<crate::identifiers::RequestId>>(
        self,
        reqi: R,
    ) -> impl Into<crate::Packet> + std::fmt::Debug {
        Isi {
            reqi: reqi.into(),
            flags: self,
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{io::Cursor, time::Duration};

    use insim_core::binrw::BinWrite;

    use super::Isi;
    use crate::{identifiers::RequestId, VERSION};

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

        let mut iname = Isi::DEFAULT_INAME.as_bytes().to_owned();
        iname.put_bytes(0, 16 - iname.len());

        assert_eq!(&buf[26..42], &iname,);
    }
}
