use bytes::{Buf, BufMut};
use insim_core::{
    identifiers::{ConnectionId, PlayerId, RequestId},
    prelude::*,
    ser::Limit,
    string::codepages,
    DecodableError, EncodableError,
};

#[cfg(feature = "serde")]
use serde::Serialize;

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
/// InsIm Info -  a /i message from user to hosts Insim
pub struct Iii {
    // pad_after_bytes = 1
    pub reqi: RequestId,

    pub ucid: ConnectionId,

    // pad_after_bytes = 2
    pub plid: PlayerId,

    pub msg: String,
}

impl Encodable for Iii {
    fn encode(&self, buf: &mut bytes::BytesMut, limit: Option<Limit>) -> Result<(), EncodableError>
    where
        Self: Sized,
    {
        if limit.is_some() {
            return Err(EncodableError::UnexpectedLimit(format!(
                "III does not support a limit: {limit:?}",
            )));
        }

        self.reqi.encode(buf, None)?;

        buf.put_bytes(0, 1);

        self.ucid.encode(buf, None)?;
        self.plid.encode(buf, None)?;

        buf.put_bytes(0, 2);

        let msg = codepages::to_lossy_bytes(&self.msg);
        buf.put_slice(&msg);

        // pad so that msg is divisible by 8
        if msg.len() % 8 != 0 {
            buf.put_bytes(0, msg.len() + 8 - (msg.len() - 8));
        }

        Ok(())
    }
}

impl Decodable for Iii {
    fn decode(buf: &mut bytes::BytesMut, limit: Option<Limit>) -> Result<Self, DecodableError>
    where
        Self: Default,
    {
        if limit.is_some() {
            return Err(DecodableError::UnexpectedLimit(format!(
                "III does not support a limit: {:?}",
                limit
            )));
        }

        let mut data = Self {
            reqi: RequestId::decode(buf, None)?,
            ..Default::default()
        };

        buf.advance(1);

        data.ucid = ConnectionId::decode(buf, None)?;
        data.plid = PlayerId::decode(buf, None)?;

        buf.advance(2);

        data.msg = String::decode(buf, Some(Limit::Bytes(buf.len())))?;

        Ok(data)
    }
}
