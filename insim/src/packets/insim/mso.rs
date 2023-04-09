use bytes::BufMut;
use insim_core::{
    identifiers::{ConnectionId, PlayerId, RequestId},
    prelude::*,
    ser::Limit,
    string::codepages,
    EncodableError,
};

#[cfg(feature = "serde")]
use serde::Serialize;

/// Enum for the sound field of [Mso].
#[derive(Debug, Default, InsimEncode, InsimDecode, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[repr(u8)]
pub enum MsoUserType {
    /// System message.
    #[default]
    System = 0,

    /// Normal, visible, user message.
    User = 1,

    /// Was this message received with the prefix character from the [Init](super::Init) message?
    Prefix = 2,

    // FIXME: Due to be retired in Insim v9
    O = 3,
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

    pub msg: String,
}

impl Encodable for Mso {
    fn encode(&self, buf: &mut bytes::BytesMut, limit: Option<Limit>) -> Result<(), EncodableError>
    where
        Self: Sized,
    {
        if limit.is_some() {
            return Err(EncodableError::UnexpectedLimit(format!(
                "MSO does not support limit! {limit:?}",
            )));
        }

        self.reqi.encode(buf, None)?;
        buf.put_bytes(0, 1);
        self.ucid.encode(buf, None)?;
        self.plid.encode(buf, None)?;
        self.usertype.encode(buf, None)?;
        self.textstart.encode(buf, None)?;

        let msg = codepages::to_lossy_bytes(&self.msg);
        buf.put_slice(&msg);

        // pad so that msg is divisible by 8
        let round_to = (msg.len() + 7) & !7;

        if round_to != msg.len() {
            buf.put_bytes(0, round_to - msg.len());
        }

        Ok(())
    }
}

impl Decodable for Mso {
    fn decode(
        buf: &mut bytes::BytesMut,
        _limit: Option<Limit>,
    ) -> Result<Self, insim_core::DecodableError>
    where
        Self: Default,
    {
        let mut data = Self {
            reqi: RequestId::decode(buf, None)?,
            ..Default::default()
        };

        buf.advance(1);
        data.ucid = ConnectionId::decode(buf, None)?;
        data.plid = PlayerId::decode(buf, None)?;
        data.usertype = MsoUserType::decode(buf, None)?;
        data.msg = String::decode(buf, Some(Limit::Bytes(buf.len())))?;
        Ok(data)
    }
}

#[cfg(test)]
mod tests {

    use bytes::{BufMut, BytesMut};
    use insim_core::Encodable;

    use super::{Mso, MsoUserType};
    use crate::core::identifiers::{ConnectionId, PlayerId, RequestId};

    #[test]
    fn dynamic_encodes_to_multiple_of_8() {
        let data = Mso {
            reqi: RequestId(1),
            ucid: ConnectionId(10),
            plid: PlayerId(74),
            usertype: MsoUserType::System,
            textstart: 0,
            msg: "two".into(),
        };

        let mut buf = BytesMut::new();
        let res = data.encode(&mut buf, None);
        assert!(res.is_ok());

        let mut comparison = BytesMut::new();
        comparison.put_u8(1);
        comparison.put_u8(0);
        comparison.put_u8(10);
        comparison.put_u8(74);
        comparison.put_u8(0);
        comparison.put_u8(0);
        comparison.extend_from_slice(&"two".to_string().as_bytes());
        comparison.put_bytes(0, 5);

        assert_eq!(buf.to_vec(), comparison.to_vec());
    }
}
