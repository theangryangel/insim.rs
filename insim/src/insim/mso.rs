use bytes::{Buf, BufMut};
use insim_core::{Decode, DecodeString, Encode, EncodeString, string::codepages};

use crate::identifiers::{ConnectionId, PlayerId, RequestId};

/// Enum for the sound field of [Mso].
#[derive(
    Debug, Default, Clone, Eq, PartialEq, PartialOrd, Ord, insim_core::Decode, insim_core::Encode,
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[repr(u8)]
#[non_exhaustive]
pub enum MsoUserType {
    /// System message.
    #[default]
    System = 0,

    /// Normal, visible, user message.
    User = 1,

    /// Was this message received with the prefix character from the [Isi](super::Isi) message?
    Prefix = 2,

    /// Hidden message (due to be retired in Insim v9?)
    O = 3,
}

const MSO_MSG_MAX_LEN: usize = 128;
const MSO_MSG_ALIGN: usize = 4;

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// System messages and user messages, variable sized.
pub struct Mso {
    /// Non-zero if the packet is a packet request or a reply to a request
    pub reqi: RequestId,

    /// Unique connection id
    pub ucid: ConnectionId,

    /// Unique player id
    pub plid: PlayerId,

    /// Set if typed by a user
    pub usertype: MsoUserType,

    /// Index of the first character of user entered text, in msg field.
    pub textstart: u8,

    /// Message
    pub msg: String,
}

impl Mso {
    /// Return msg with the textstart stripped
    pub fn msg_from_textstart(&self) -> &str {
        &self.msg[self.textstart as usize..]
    }
}

impl Decode for Mso {
    fn decode(buf: &mut bytes::Bytes) -> Result<Self, insim_core::DecodeError> {
        let reqi = RequestId::decode(buf)?;
        buf.advance(1);
        let ucid = ConnectionId::decode(buf)?;
        let plid = PlayerId::decode(buf)?;
        let usertype = MsoUserType::decode(buf)?;
        let textstart = u8::decode(buf)?;

        let (textstart, msg) = if textstart > 0 {
            let mut name = buf.split_to(textstart as usize);
            let name_len = name.len();
            let name = String::decode_codepage(&mut name, name_len)?;
            let msg = String::decode_codepage(buf, buf.len())?;
            (name.len() as u8, format!("{name}{msg}"))
        } else {
            (0_u8, String::decode_codepage(buf, buf.len())?)
        };

        Ok(Self {
            reqi,
            ucid,
            plid,
            usertype,
            textstart,
            msg,
        })
    }
}

impl Encode for Mso {
    fn encode(&self, buf: &mut bytes::BytesMut) -> Result<(), insim_core::EncodeError> {
        self.reqi.encode(buf)?;
        buf.put_bytes(0, 1);
        self.ucid.encode(buf)?;
        self.plid.encode(buf)?;
        self.usertype.encode(buf)?;

        if self.textstart > 0 {
            let name = &self.msg[..self.textstart as usize];
            let msg = &self.msg[(self.textstart as usize)..];

            let name = codepages::to_lossy_bytes(name);
            let msg = codepages::to_lossy_bytes(msg);

            if (name.len() + msg.len()) > (MSO_MSG_MAX_LEN - 1) {
                return Err(insim_core::EncodeErrorKind::OutOfRange { min: 0, max: MSO_MSG_ALIGN, found: name.len() + msg.len() }.context("Mso name and msg out of range"));
            }

            let textstart = name.len() as u8;

            buf.put_u8(textstart);

            let mut remaining = MSO_MSG_MAX_LEN;

            let name_len_to_write = name.len().min(remaining);
            buf.extend_from_slice(&name[..name_len_to_write]);
            remaining -= name_len_to_write;

            let msg_len_to_write = msg.len().min(remaining);
            buf.extend_from_slice(&msg[..msg_len_to_write]);

            let written = name_len_to_write + msg_len_to_write;
            if remaining > 0 {
                let align_to = MSO_MSG_ALIGN - 1;
                let round_to = (written + align_to) & !align_to;
                let round_to = round_to.min(MSO_MSG_MAX_LEN);
                buf.put_bytes(0, round_to - written);
            }
        } else {
            buf.put_u8(0);
            self.msg
                .encode_codepage_with_alignment(buf, MSO_MSG_MAX_LEN, MSO_MSG_ALIGN, true)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use bytes::{BufMut, Bytes, BytesMut};

    use super::*;

    #[test]
    fn test_mso() {
        let mut comparison = BytesMut::new();
        comparison.put_u8(1); // reqi
        comparison.put_u8(0);
        comparison.put_u8(10); // ucid
        comparison.put_u8(74); // plid
        comparison.put_u8(0); // usertype
        comparison.put_u8(0); // textstart
        comparison.extend_from_slice(&"two".to_string().as_bytes()); // msg
        comparison.put_bytes(0, 1);

        assert_from_to_bytes!(Mso, comparison.as_ref(), |parsed: Mso| {
            assert_eq!(parsed.reqi, RequestId(1));
            assert_eq!(parsed.ucid, ConnectionId(10));
            assert_eq!(parsed.plid, PlayerId(74));
            assert_eq!(parsed.usertype, MsoUserType::System);
            assert_eq!(parsed.textstart, 0);
            assert_eq!(parsed.msg, "two");
        });
    }

    #[test]
    fn test_mso_too_short() {
        let mut raw = BytesMut::new();
        raw.put_u8(0); // reqi
        raw.put_u8(0);
        raw.put_u8(10); // ucid
        raw.put_u8(74); // plid
        raw.put_u8(0); // usertype
        raw.put_u8(0); // textstart
        raw.extend_from_slice(&"Downloaded Skin : XFG_PRO38".to_string().as_bytes()); // ms
        // We are intentionally dropping the trailing nul byte here to ensure that we handle
        // packets that are too short somehow. For this reason we're not using
        // assert_from_to_bytes!
        //raw.put_bytes(0, 1);
        let raw = raw.freeze();

        let res = Mso::decode(&mut Bytes::from(raw.clone())).unwrap();
        assert_eq!(res.textstart, 0);
        assert_eq!(res.msg, "Downloaded Skin : XFG_PRO38");
    }

    #[test]
    fn test_mso_too_long() {
        let mut raw = BytesMut::new();
        raw.put_u8(0); // reqi
        raw.put_u8(0);
        raw.put_u8(10); // ucid
        raw.put_u8(74); // plid
        raw.put_u8(0); // usertype
        raw.put_u8(0); // textstart
        raw.extend_from_slice(&[b'X'; MSO_MSG_MAX_LEN + 10]); // msg
        let raw = raw.freeze();

        // when reading we want to handle too long entries, but ensure that when we convert to
        // bytes it's appropriately truncated

        let res = Mso::decode(&mut Bytes::from(raw.clone())).unwrap();
        assert_eq!(res.textstart, 0);
        assert_eq!(res.msg.len(), MSO_MSG_MAX_LEN + 10);

        let mut buf = BytesMut::new();
        let res = res.encode(&mut buf);
        assert!(res.is_err());
    }

    #[test]
    fn test_codepages_moves_textstart() {
        let raw = [
            0, 0, 2, 4, 1, 17, 94, // msg
            55, 80, 108, 97, 121, 101, 114, 32, 94, 69, 236, 32, 94, 55, 58, 32, 94, 56, 99, 114,
            154, 94, 69, 232, 0, 0, 0,
        ];

        assert_from_to_bytes!(Mso, raw, |mso: Mso| {
            // Shamefully borrowed from https://github.com/simbroadcasts/node-insim/commit/533d107b695b58df64278a5935a7140fa340fb3d
            assert_eq!(mso.msg, "^7Player ě ^7: ^8cršč");
            assert_eq!(mso.textstart, 16); // moved from 17th position to 16th
            assert_eq!(&mso.msg[..mso.textstart as usize], "^7Player ě ^7: ");
            assert_eq!(&mso.msg[mso.textstart as usize..], "^8cršč");
        });
    }
}
