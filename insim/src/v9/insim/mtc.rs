use insim_core::{
    identifiers::{ConnectionId, PlayerId, RequestId},
    prelude::*,
    ser::Limit,
    string::codepages,
    EncodableError,
};

pub use super::SoundType;

#[cfg(feature = "serde")]
use serde::Serialize;

#[derive(Debug, InsimDecode, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
/// Message to Connection - Send a message to a specific connection, restricted to hosts only
pub struct Mtc {
    pub reqi: RequestId,
    pub sound: SoundType,

    pub ucid: ConnectionId,
    #[insim(pad_bytes_after = "2")]
    pub plid: PlayerId,

    #[insim(bytes = "128")]
    pub msg: String,
}

impl Encodable for Mtc {
    fn encode(
        &self,
        buf: &mut bytes::BytesMut,
        limit: Option<Limit>,
    ) -> Result<(), EncodableError> {
        if limit.is_some() {
            return Err(EncodableError::UnexpectedLimit(format!(
                "Mtc does not support a limit: {limit:?}",
            )));
        }

        self.reqi.encode(buf, None)?;
        self.sound.encode(buf, None)?;

        self.ucid.encode(buf, None)?;
        self.plid.encode(buf, None)?;

        buf.put_bytes(0, 2);

        let msg = codepages::to_lossy_bytes(&self.msg);

        if msg.len() > 127 {
            return Err(EncodableError::WrongSize(
                "Mtc packet only supports up to 127 character messages".into(),
            ));
        }

        buf.put_slice(&msg);

        // pad so that msg is divisible by 4
        let round_to = (msg.len() + 3) & !3;

        if round_to != msg.len() {
            buf.put_bytes(0, round_to - msg.len());
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use bytes::BytesMut;
    use insim_core::{
        identifiers::{ConnectionId, PlayerId},
        Encodable,
    };

    use super::{Mtc, SoundType};
    use crate::core::identifiers::RequestId;

    #[test]
    fn ensure_last_byte_zero_always() {
        let data = Mtc {
            reqi: RequestId(1),
            plid: PlayerId(0),
            ucid: ConnectionId(0),
            sound: SoundType::default(),
            msg: "aaaaa".into(),
        };

        let mut buf = BytesMut::new();
        let res = data.encode(&mut buf, None);
        assert!(res.is_ok());

        assert_eq!((buf.len() - 6) % 4, 0);

        assert_eq!(buf.last(), Some(&0));
        assert_eq!(buf.len(), 14);
    }
}
