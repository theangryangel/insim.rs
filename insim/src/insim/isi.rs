use std::time::Duration;

use bitflags::bitflags;

use crate::{identifiers::RequestId, WithRequestId, VERSION};

bitflags! {
    /// Flags for the [Init] packet flags field.
    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy, Default)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize))]
    /// Flags for [Isi], used to indicate what behaviours we want to opt into
    pub struct IsiFlags: u16 {
        /// Guest or single player
        const LOCAL = (1_u16 << 2);

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

impl_bitflags_from_to_bytes!(IsiFlags, u16);

#[derive(Debug, Clone, Eq, PartialEq, insim_core::Decode, insim_core::Encode)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "python", pyo3::prelude::pyclass)]
/// Insim Init, or handshake packet.
/// Required to be sent to the server before any other packets.
pub struct Isi {
    /// When set to a non-zero value the server will send a [crate::Packet::Ver] packet in response.
    ///packet in response.
    #[insim(pad_after = 1)]
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
    pub prefix: char,

    /// Time in between each [Nlp](super::Nlp) or [Mci](super::Mci) packet when set to a non-zero value and
    /// the relevant flags are set.
    #[insim(duration(milliseconds = u16))]
    pub interval: Duration,

    /// Administrative password.
    #[insim(codepage(length = 16))]
    pub admin: String,

    /// Name of the program.
    #[insim(codepage(length = 16, trailing_nul = true))]
    pub iname: String,
}

impl Isi {
    /// Default application name
    pub const DEFAULT_INAME: &'static str = "insim.rs";
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
    use super::*;

    #[test]
    fn test_isi_flags() {
        let local = IsiFlags::from_bits_truncate(4);
        assert!(local.contains(IsiFlags::LOCAL));
    }

    #[test]
    fn test_isi() {
        assert_from_to_bytes!(
            Isi,
            [
                1, // reqi
                0, 1,       // udpport (1)
                1,       // udpport (2)
                4,       // flags (1)
                0,       // flags (2)
                VERSION, // insimver
                b'!',    // prefix
                1,       // interval (1)
                0,       // interval (2)
                b'a', b'd', b'm', b'i', b'n', b'|', b'*', b':', b'\\', b'/', b'?', b'"', b'<',
                b'>', b'#', 0, b'i', b'n', b's', b'i', b'm', b'.', b'r', b's', 0, 0, 0, 0, 0, 0, 0,
                0
            ],
            |data: Isi| {
                assert_eq!(data.reqi, RequestId(1));
                assert_eq!(&data.iname, "insim.rs");
                assert_eq!(&data.admin, "admin|*:\\/?\"<>#");
                assert_eq!(data.version, VERSION);
                assert_eq!(data.prefix, '!');
                assert_eq!(data.interval, Duration::from_millis(1));
                assert!(data.flags.contains(IsiFlags::LOCAL));
            }
        );
    }
}
