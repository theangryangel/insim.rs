use bytes::{Buf, BufMut};
use insim_core::{
    identifiers::{ConnectionId, PlayerId, RequestId},
    prelude::*,
    string::CodepageString,
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

    pub msg: CodepageString,
}

impl Encodable for Iii {
    fn encode(&self, buf: &mut bytes::BytesMut) -> Result<(), insim_core::EncodableError>
    where
        Self: Sized,
    {
        self.reqi.encode(buf)?;

        buf.put_bytes(0, 1)?;

        self.ucid.encode(buf)?;
        self.plid.encode(buf)?;

        buf.put_bytes(0, 2)?;

        let msg = self.msg.into_bytes();
        buf.put(msg);

        // pad so that msg is divisible by 8
        if msg.len() % 8 != 0 {
            buf.put_bytes(0, msg.len() + 8 - (msg.len() - 8));
        }

        Ok(())
    }
}

impl Decodable for Iii {
    fn decode(
        buf: &mut bytes::BytesMut,
        count: Option<usize>,
    ) -> Result<Self, insim_core::DecodableError>
    where
        Self: Default,
    {
        let data = Self::default();

        data.reqi = RequestId::decode(buf, count)?;
        buf.advance(1)?;

        data.ucid = ConnectionId::decode(buf, count)?;
        data.plid = PlayerId::decode(buf, count)?;

        buf.advance(2)?;

        data.msg = CodepageString::decode(buf, count)?;

        Ok(data)
    }
}
