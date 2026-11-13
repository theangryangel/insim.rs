use bytes::BufMut;
use insim_core::{Decode, DecodeContext, Encode, EncodeContext, string::codepages};

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
    fn decode(ctx: &mut DecodeContext) -> Result<Self, insim_core::DecodeError> {
        let reqi = ctx.decode::<RequestId>("reqi")?;
        let mso_data = ctx.decode::<u8>("mso_data")?;
        // Unknown identifiers fall back to Latin, as do pre-0.8C27 packets.
        let codepage = codepages::Codepage::try_from(mso_data & 0x0f).unwrap_or_default();
        let ucid = ctx.decode::<ConnectionId>("ucid")?;
        let plid = ctx.decode::<PlayerId>("plid")?;
        let usertype = ctx.decode::<MsoUserType>("usertype")?;
        let textstart = ctx.decode::<u8>("textstart")?;

        let raw = ctx.buf.clone();
        if usize::from(textstart) > raw.len() {
            return Err(
                insim_core::DecodeError::from(insim_core::DecodeErrorKind::UnexpectedEof)
                    .context("Mso::textstart"),
            );
        }
        // Decode the prefix to translate the wire byte offset into a UTF-8 byte offset.
        // Decode the full message together so code page switches carry across the boundary.
        let prefix =
            codepages::to_lossy_string_with_codepage(&raw[..usize::from(textstart)], codepage);
        let textstart = u8::try_from(prefix.len()).map_err(|_| {
            insim_core::DecodeErrorKind::OutOfRange {
                min: 0,
                max: usize::from(u8::MAX),
                found: prefix.len(),
            }
            .context("Mso::textstart")
        })?;
        let msg_len = raw.len();
        let msg = ctx.decode_codepage_with_codepage("msg", msg_len, codepage)?;

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
    fn encode(&self, ctx: &mut EncodeContext) -> Result<(), insim_core::EncodeError> {
        ctx.encode("reqi", &self.reqi)?;
        // Encoding uses Latin as the default, regardless of the received code page.
        ctx.encode("mso_data", &u8::from(codepages::Codepage::Latin))?;
        ctx.encode("ucid", &self.ucid)?;
        ctx.encode("plid", &self.plid)?;
        ctx.encode("usertype", &self.usertype)?;

        if self.textstart > 0 {
            let name = &self.msg[..self.textstart as usize];
            let msg = &self.msg[(self.textstart as usize)..];

            let mut name = codepages::to_lossy_bytes(name);
            let msg = codepages::to_lossy_bytes(msg);

            // The body is encoded independently with a Latin default. Restore that
            // state at the boundary if the name switched to another code page.
            let mut active = codepages::Codepage::Latin;
            let mut i = 0;
            while i + 1 < name.len() {
                if name[i] == b'^' {
                    if name[i + 1] == b'8' {
                        active = codepages::Codepage::Latin;
                    } else if let Ok(cp) = codepages::Codepage::try_from(char::from(name[i + 1])) {
                        active = cp;
                    }
                    i += 2;
                } else {
                    i += 1;
                }
            }
            let body_sets_codepage = msg.len() >= 2
                && msg[0] == b'^'
                && (msg[1] == b'8' || codepages::Codepage::try_from(char::from(msg[1])).is_ok());
            if active != codepages::Codepage::Latin && !body_sets_codepage {
                name.to_mut().extend_from_slice(b"^L");
            }

            if (name.len() + msg.len()) > (MSO_MSG_MAX_LEN - 1) {
                return Err(insim_core::EncodeErrorKind::OutOfRange {
                    min: 0,
                    max: MSO_MSG_ALIGN,
                    found: name.len() + msg.len(),
                }
                .context("Mso: name + msg"));
            }

            let textstart = name.len() as u8;

            ctx.buf.put_u8(textstart);

            let mut remaining = MSO_MSG_MAX_LEN;

            let name_len_to_write = name.len().min(remaining);
            ctx.buf.extend_from_slice(&name[..name_len_to_write]);
            remaining -= name_len_to_write;

            let msg_len_to_write = msg.len().min(remaining);
            ctx.buf.extend_from_slice(&msg[..msg_len_to_write]);

            let written = name_len_to_write + msg_len_to_write;
            if remaining > 0 {
                let align_to = MSO_MSG_ALIGN - 1;
                let round_to = (written + align_to) & !align_to;
                let round_to = round_to.min(MSO_MSG_MAX_LEN);
                ctx.buf.put_bytes(0, round_to - written);
            }
        } else {
            ctx.buf.put_u8(0);
            ctx.encode_codepage("msg", &self.msg, MSO_MSG_MAX_LEN, Some(MSO_MSG_ALIGN), true)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use bytes::{BufMut, BytesMut};
    use insim_core::{DecodeContext, EncodeContext};

    use super::*;

    #[test]
    fn unsplit_non_latin_message() {
        let mut raw = bytes::Bytes::from_static(b"\0\x05\0\0\0\0\xff^8\xff\0");
        let mso = Mso::decode(&mut DecodeContext::new(&mut raw)).unwrap();
        assert_eq!(mso.msg, "я^8я");
        assert_eq!(mso.msg_from_textstart(), "я^8я");
    }

    #[test]
    fn non_latin_mso_uses_low_nibble_and_reencodes_as_latin() {
        let mut raw = bytes::Bytes::from_static(b"\0\xa1\x02\x04\x01\x04\xec : \xe8^L\xff^8\xec\0");
        let mso = Mso::decode(&mut DecodeContext::new(&mut raw)).unwrap();
        assert_eq!(mso.msg, "ě : čÿ^8ě");
        assert_eq!(mso.textstart, 5);
        assert_eq!(mso.msg_from_textstart(), "čÿ^8ě");
        let mut encoded = BytesMut::new();
        mso.encode(&mut EncodeContext::new(&mut encoded)).unwrap();
        assert_eq!(encoded[1], 0);
        let decoded = Mso::decode(&mut DecodeContext::new(&mut encoded.freeze())).unwrap();
        assert_eq!(decoded.msg, mso.msg);
        assert_eq!(decoded.textstart, mso.textstart);
    }

    #[test]
    fn codepage_switch_in_name_carries_into_body() {
        let mut raw = bytes::Bytes::from_static(b"\0\0\x02\x04\x01\x06^E\xec : \xe8\0");
        let mso = Mso::decode(&mut DecodeContext::new(&mut raw)).unwrap();
        assert_eq!(mso.msg, "ě : č");
        assert_eq!(mso.msg_from_textstart(), "č");
    }

    #[test]
    fn independently_encoded_body_restores_latin_after_name() {
        let mso = Mso {
            msg: "ě : ÿ".into(),
            textstart: 5,
            ..Default::default()
        };
        let mut encoded = BytesMut::new();
        mso.encode(&mut EncodeContext::new(&mut encoded)).unwrap();
        let decoded = Mso::decode(&mut DecodeContext::new(&mut encoded.freeze())).unwrap();
        assert_eq!(decoded.msg, mso.msg);
        assert_eq!(decoded.msg_from_textstart(), "ÿ");
    }

    #[test]
    fn unknown_codepage_falls_back_to_latin() {
        let mut raw = bytes::Bytes::from_static(b"\0\x0f\0\0\0\0\xff\0");
        let mso = Mso::decode(&mut DecodeContext::new(&mut raw)).unwrap();
        assert_eq!(mso.msg, "ÿ");
    }

    #[test]
    fn test_mso() {
        let mut comparison = BytesMut::new();
        comparison.put_u8(1); // reqi
        comparison.put_u8(0);
        comparison.put_u8(10); // ucid
        comparison.put_u8(74); // plid
        comparison.put_u8(0); // usertype
        comparison.put_u8(0); // textstart
        comparison.extend_from_slice("two".to_string().as_bytes()); // msg
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
        raw.extend_from_slice("Downloaded Skin : XFG_PRO38".to_string().as_bytes()); // ms
        // We are intentionally dropping the trailing nul byte here to ensure that we handle
        // packets that are too short somehow. For this reason we're not using
        // assert_from_to_bytes!
        //raw.put_bytes(0, 1);
        let raw = raw.freeze();

        let mut buf = raw.clone();
        let res = Mso::decode(&mut DecodeContext::new(&mut buf)).unwrap();
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

        let mut bytes = raw.clone();
        let res = Mso::decode(&mut DecodeContext::new(&mut bytes)).unwrap();
        assert_eq!(res.textstart, 0);
        assert_eq!(res.msg.len(), MSO_MSG_MAX_LEN + 10);

        let mut buf = BytesMut::new();
        let res = res.encode(&mut EncodeContext::new(&mut buf));
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
