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
    pub reqi: RequestId,

    pub ucid: ConnectionId,
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

        if msg.len() > 63 {
            return Err(EncodableError::WrongSize(
                "III packet only supports up to 63 character messages".into(),
            ));
        }

        buf.put_slice(&msg);

        // last byte is always 0
        buf.put_bytes(0, 1);

        // pad so that msg is divisible by 4
        // after the size and type are added later
        let total = msg.len() + 2;
        let round_to = (total + 3) & !3;
        if round_to != total {
            buf.put_bytes(0, round_to - total);
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
