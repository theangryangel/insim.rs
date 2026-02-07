use std::time::Duration;

use bitflags::bitflags;

use crate::{VERSION, WithRequestId, identifiers::RequestId};

bitflags! {
    /// Flags for the [Isi] options field.
    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy, Default)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    /// Flags for [Isi], used to indicate which updates and behaviours to enable.
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

        /// Receive AXM when loading a layout
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
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// InSim init/handshake packet sent to start communication.
///
/// - Sends initial connection options (which updates you want and how often).
/// - Controls where updates are delivered when using UDP.
/// - Requests a version reply when `reqi` is non-zero.
pub struct Isi {
    /// If non-zero, requests a [crate::insim::Ver] reply and is echoed back.
    #[insim(pad_after = 1)]
    pub reqi: RequestId,

    /// UDP port for replies.
    /// - UDP: `0` = reply to the source port; non-zero = reply to this port.
    /// - TCP: `0` = NLP/MCI over TCP; non-zero = NLP/MCI over UDP.
    pub udpport: u16,

    /// Connection options (e.g., NLP/MCI updates, collision reports, join handling).
    ///
    /// Prefer [Mci](super::Mci) over [Nlp](super::Nlp) as it includes all NLP data.
    pub flags: IsiFlags,

    /// Protocol version your program expects.
    /// Defaults to [crate::VERSION].
    pub version: u8,

    /// Special host message prefix (single ASCII character).
    /// Messages typed with this prefix are forwarded to InSim and not shown in chat.
    pub prefix: char,

    /// Interval between [Nlp](super::Nlp) or [Mci](super::Mci) updates (0 = disabled).
    #[insim(duration = u16)]
    pub interval: Duration,

    /// Admin password (empty if none).
    #[insim(codepage(length = 16))]
    pub admin: String,

    /// Short program name shown to LFS.
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
