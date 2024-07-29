use std::io::SeekFrom;

use bytes::BufMut;
use insim_core::{
    binrw::{self, binrw},
    string::codepages,
};

use crate::identifiers::{ConnectionId, PlayerId, RequestId};

/// Enum for the sound field of [Mso].
#[binrw]
#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[repr(u8)]
#[brw(repr(u8))]
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

impl binrw::BinRead for Mso {
    type Args<'a> = ();

    fn read_options<R: std::io::prelude::Read + std::io::prelude::Seek>(
        reader: &mut R,
        endian: binrw::Endian,
        _args: Self::Args<'_>,
    ) -> binrw::prelude::BinResult<Self> {
        let reqi = RequestId::read_options(reader, endian, ())?;

        let _ = reader.seek(SeekFrom::Current(1))?;

        let ucid = ConnectionId::read_options(reader, endian, ())?;
        let plid = PlayerId::read_options(reader, endian, ())?;
        let usertype = MsoUserType::read_options(reader, endian, ())?;
        let textstart = u8::read_options(reader, endian, ())?;
        let (textstart, msg) = if textstart > 0 {
            let name = Vec::<u8>::read_options(
                reader,
                endian,
                binrw::VecArgs {
                    count: textstart as usize,
                    inner: (),
                },
            )?;

            let msg: Vec<u8> = binrw::helpers::until_eof(reader, endian, ())?;

            let name = codepages::to_lossy_string(&name);
            let msg = codepages::to_lossy_string(&msg);
            (name.len() as u8, format!("{name}{msg}"))
        } else {
            let msg: Vec<u8> = binrw::helpers::until_eof(reader, endian, ())?;
            (0_u8, codepages::to_lossy_string(&msg).to_string())
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

impl binrw::BinWrite for Mso {
    type Args<'a> = ();

    fn write_options<W: std::io::prelude::Write + std::io::prelude::Seek>(
        &self,
        writer: &mut W,
        endian: binrw::Endian,
        _args: Self::Args<'_>,
    ) -> binrw::prelude::BinResult<()> {
        self.reqi.write_options(writer, endian, ())?;
        0_u8.write_options(writer, endian, ())?; // pad 1 byte
        self.ucid.write_options(writer, endian, ())?;
        self.plid.write_options(writer, endian, ())?;
        self.usertype.write_options(writer, endian, ())?;

        // if we need to encode the string, we need to move the textstart transparently for the
        // user
        let textstart = if self.textstart > 0 {
            let name = &self.msg[..self.textstart as usize];
            let textstart = codepages::to_lossy_bytes(name).len();

            textstart as u8
        } else {
            self.textstart
        };

        textstart.write_options(writer, endian, ())?;
        let mut res = codepages::to_lossy_bytes(&self.msg).to_vec();

        let align_to = MSO_MSG_ALIGN - 1;
        let round_to = (res.len() + align_to) & !align_to;
        if round_to != res.len() {
            res.put_bytes(0, round_to - res.len());
        }
        res.truncate(MSO_MSG_MAX_LEN);
        res.write_options(writer, endian, ())?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use bytes::{BufMut, BytesMut};
    use insim_core::binrw::{BinRead, BinWrite};
    use tokio_test::assert_ok;

    use super::*;

    #[test]
    fn test_mso() {
        let data = Mso {
            reqi: RequestId(1),
            ucid: ConnectionId(10),
            plid: PlayerId(74),
            usertype: MsoUserType::System,
            textstart: 0,
            msg: "two".into(),
        };

        let mut buf = Cursor::new(Vec::new());
        let res = data.write_le(&mut buf);
        assert!(res.is_ok());

        let mut comparison = BytesMut::new();
        comparison.put_u8(1);
        comparison.put_u8(0);
        comparison.put_u8(10);
        comparison.put_u8(74);
        comparison.put_u8(0);
        comparison.put_u8(0);
        comparison.extend_from_slice(&"two".to_string().as_bytes());
        comparison.put_bytes(0, 1);

        assert_eq!(buf.into_inner(), comparison.to_vec());
    }

    #[test]
    fn test_mso_too_short() {
        let mut buf = Cursor::new(b"\x0b\0\0\0\0\0\0Downloaded Skin : XFG_PRO38\0");

        let res = Mso::read_le(&mut buf);
        assert_ok!(res);
    }

    #[test]
    fn test_codepages_moves_textstart() {
        let mut buf = Cursor::new([
            0, 0, 2, 4, 1, 17, 94, 55, 80, 108, 97, 121, 101, 114, 32, 94, 69, 236, 32, 94, 55, 58,
            32, 94, 56, 99, 114, 154, 94, 69, 232, 0, 0, 0,
        ]);

        let mso = Mso::read_le(&mut buf).unwrap();
        // Shamefully borrowed from https://github.com/simbroadcasts/node-insim/commit/533d107b695b58df64278a5935a7140fa340fb3d
        assert_eq!(mso.msg, "^7Player ě ^7: ^8cršč");
        assert_eq!(mso.textstart, 16); // moved from 17th position to 16th
        assert_eq!(&mso.msg[..mso.textstart as usize], "^7Player ě ^7: ");
        assert_eq!(&mso.msg[mso.textstart as usize..], "^8cršč");
    }
}
