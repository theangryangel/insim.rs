use insim_core::{identifiers::RequestId, prelude::*, string::codepages, EncodableError};

#[cfg(feature = "serde")]
use serde::Serialize;

#[derive(Debug, InsimDecode, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
/// Message Type - Send a message to LFS as if typed by a user
pub struct Mst {
    #[insim(pad_bytes_after = "1")]
    pub reqi: RequestId,

    #[insim(bytes = "64")]
    pub msg: String,
}

impl Encodable for Mst {
    fn encode(
        &self,
        buf: &mut bytes::BytesMut,
        limit: Option<insim_core::ser::Limit>,
    ) -> Result<(), insim_core::EncodableError> {
        if limit.is_some() {
            return Err(EncodableError::UnexpectedLimit(format!(
                "Mst does not support limit! {limit:?}",
            )));
        }

        self.reqi.encode(buf, None)?;
        buf.put_bytes(0, 1);

        let msg = codepages::to_lossy_bytes(&self.msg);
        if msg.len() > 63 {
            return Err(EncodableError::WrongSize(
                "Mst only supports upto 63 byte long messages".into(),
            ));
        }
        msg.encode(buf, Some(insim_core::ser::Limit::Bytes(63)))?;
        // last byte must be zero
        if msg.len() < 64 {
            buf.put_bytes(0, 64 - msg.len());
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use bytes::BytesMut;
    use insim_core::Encodable;

    use super::Mst;
    use crate::core::identifiers::RequestId;

    #[test]
    fn ensure_last_byte_zero_always() {
        let data = Mst {
            reqi: RequestId(1),
            msg: "aaaaaa".into(),
        };

        let mut buf = BytesMut::new();
        let res = data.encode(&mut buf, None);
        assert!(res.is_ok());

        assert_eq!(buf.last(), Some(&0));
        assert_eq!(buf.len(), 66);
    }
}
