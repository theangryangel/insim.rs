use bytes::BufMut;
use insim_core::{
    identifiers::{ConnectionId, PlayerId, RequestId},
    prelude::*,
    ser::Limit,
    string::CodepageString,
    EncodableError,
};

#[cfg(feature = "serde")]
use serde::Serialize;

/// Enum for the sound field of [Mso].
#[derive(Debug, InsimEncode, InsimDecode, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[repr(u8)]
pub enum MsoUserType {
    /// System message.
    System = 0,

    /// Normal, visible, user message.
    User = 1,

    /// Was this message received with the prefix character from the [Init](super::Init) message?
    Prefix = 2,

    // FIXME: Due to be retired in Insim v9
    O = 3,
}

impl Default for MsoUserType {
    fn default() -> Self {
        MsoUserType::System
    }
}

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
/// System messsages and user messages, variable sized.
pub struct Mso {
    pub reqi: RequestId,

    pub ucid: ConnectionId,

    pub plid: PlayerId,

    /// Set if typed by a user
    pub usertype: MsoUserType,

    /// Index of the first character of user entered text, in msg field.
    pub textstart: u8,

    pub msg: CodepageString,
}

impl Encodable for Mso {
    fn encode(&self, buf: &mut bytes::BytesMut, limit: Option<Limit>) -> Result<(), EncodableError>
    where
        Self: Sized,
    {
        if limit.is_some() {
            return Err(EncodableError::UnexpectedLimit(format!(
                "MSO does not support limit! {:?}",
                limit
            )));
        }

        self.reqi.encode(buf, None)?;
        buf.put_bytes(0, 1);
        self.ucid.encode(buf, None)?;
        self.plid.encode(buf, None)?;
        self.usertype.encode(buf, None)?;
        self.textstart.encode(buf, None)?;

        let msg = self.msg.into_bytes();
        buf.put(msg);

        // pad so that msg is divisible by 8
        if msg.len() % 8 != 0 {
            buf.put_bytes(0, msg.len() + 8 - (msg.len() - 8));
        }

        Ok(())
    }
}

impl Decodable for Mso {
    fn decode(
        buf: &mut bytes::BytesMut,
        limit: Option<Limit>,
    ) -> Result<Self, insim_core::DecodableError>
    where
        Self: Default,
    {
        let mut data = Self::default();

        data.reqi = RequestId::decode(buf, None)?;
        buf.advance(1);
        data.ucid = ConnectionId::decode(buf, None)?;
        data.plid = PlayerId::decode(buf, None)?;
        data.usertype = MsoUserType::decode(buf, None)?;
        data.msg = CodepageString::decode(buf, Some(Limit::Bytes(buf.len())))?;
        Ok(data)
    }
}
