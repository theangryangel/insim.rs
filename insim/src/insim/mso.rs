use bytes::{Buf, BufMut};
use insim_core::{string::codepages, Decode, DecodeString, Encode, EncodeString};

use crate::identifiers::{ConnectionId, PlayerId, RequestId};

/// Source/type of a message reported by [Mso].
#[derive(
    Debug, Default, Clone, Eq, PartialEq, PartialOrd, Ord, insim_core::Decode, insim_core::Encode,
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(u8)]
#[non_exhaustive]
pub enum MsoUserType {
    /// System message.
    #[default]
    System = 0,

    /// Normal, visible, user message.
    User = 1,

    /// Message received with the prefix character from [Isi](super::Isi).
    Prefix = 2,

    /// Hidden message typed with the `/o` command.
    O = 3,
}

const MSO_MSG_MAX_LEN: usize = 128;
const MSO_MSG_ALIGN: usize = 4;

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// System and user messages reported by LFS.
///
/// - Variable-length packet with a message payload.
/// - `textstart` marks where user-entered text begins within `msg`.
pub struct Mso {
    /// Request identifier echoed by replies.
    pub reqi: RequestId,

    /// Connection that sent the message (0 = host).
    pub ucid: ConnectionId,

    /// Player that sent the message (0 = use `ucid`).
    pub plid: PlayerId,

    /// Message origin and visibility.
    pub usertype: MsoUserType,

    /// Index of the first user-entered character within `msg`.
    pub textstart: u8,

    /// Full message text (may include name prefix before `textstart`).
    pub msg: String,
}

impl Mso {
    /// Return the message starting from `textstart`.
    pub fn msg_from_textstart(&self) -> &str {
        &self.msg[self.textstart as usize..]
    }
}

impl Decode for Mso {
    fn decode(buf: &mut bytes::Bytes) -> Result<Self, insim_core::DecodeError> {
        let reqi = RequestId::decode(buf).map_err(|e| e.nested().context("Mso::reqi"))?;
        buf.advance(1);
        let ucid = ConnectionId::decode(buf).map_err(|e| e.nested().context("Mso::ucid"))?;
        let plid = PlayerId::decode(buf).map_err(|e| e.nested().context("Mso::plid"))?;
        let usertype = MsoUserType::decode(buf).map_err(|e| e.nested().context("Mso::usertype"))?;
        let textstart = u8::decode(buf).map_err(|e| e.nested().context("Mso::textstart"))?;

        let (textstart, msg) = if textstart > 0 {
            let mut name = buf.split_to(textstart as usize);
            let name_len = name.len();
            let name = String::decode_codepage(&mut name, name_len)
                .map_err(|e| e.nested().context("Mso::name"))?;
            let msg = String::decode_codepage(buf, buf.len())
                .map_err(|e| e.nested().context("Mso::msg"))?;
            (name.len() as u8, format!("{name}{msg}"))
        } else {
            (
                0_u8,
                String::decode_codepage(buf, buf.len())
                    .map_err(|e| e.nested().context("Mso::msg"))?,
            )
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
        self.reqi
            .encode(buf)
            .map_err(|e| e.nested().context("Mso::reqi"))?;
        buf.put_bytes(0, 1);
        self.ucid
            .encode(buf)
            .map_err(|e| e.nested().context("Mso::ucid"))?;
        self.plid
            .encode(buf)
            .map_err(|e| e.nested().context("Mso::plid"))?;
        self.usertype
            .encode(buf)
            .map_err(|e| e.nested().context("Mso::usertype"))?;

        if self.textstart > 0 {
            let name = &self.msg[..self.textstart as usize];
            let msg = &self.msg[(self.textstart as usize)..];

            let name = codepages::to_lossy_bytes(name);
            let msg = codepages::to_lossy_bytes(msg);

            if (name.len() + msg.len()) > (MSO_MSG_MAX_LEN - 1) {
                return Err(insim_core::EncodeErrorKind::OutOfRange {
                    min: 0,
                    max: MSO_MSG_ALIGN,
                    found: name.len() + msg.len(),
                }
                .context("Mso: name + msg"));
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
                .encode_codepage_with_alignment(buf, MSO_MSG_MAX_LEN, MSO_MSG_ALIGN, true)
                .map_err(|e| e.nested().context("Mso::msg"))?;
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
