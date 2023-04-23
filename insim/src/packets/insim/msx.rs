use insim_core::{identifiers::RequestId, prelude::*, string::codepages, EncodableError};

#[cfg(feature = "serde")]
use serde::Serialize;

#[derive(Debug, InsimDecode, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
/// Extended Message (like [Mst](super::Mst), but longer)
pub struct Msx {
    #[insim(pad_bytes_after = "1")]
    pub reqi: RequestId,

    #[insim(bytes = "96")]
    pub msg: String,
}

impl Encodable for Msx {
    fn encode(
        &self,
        buf: &mut bytes::BytesMut,
        limit: Option<insim_core::ser::Limit>,
    ) -> Result<(), insim_core::EncodableError> {
        if limit.is_some() {
            return Err(EncodableError::UnexpectedLimit(format!(
                "Msx does not support limit! {limit:?}",
            )));
        }

        self.reqi.encode(buf, None)?;
        buf.put_bytes(0, 1);

        let msg = codepages::to_lossy_bytes(&self.msg);
        if msg.len() > 95 {
            return Err(EncodableError::WrongSize(
                "Msx only supports upto 95 byte long messages".into(),
            ));
        }
        msg.encode(buf, Some(insim_core::ser::Limit::Bytes(95)))?;
        // last byte must be zero
        if msg.len() < 96 {
            buf.put_bytes(0, 96 - msg.len());
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use bytes::BytesMut;
    use insim_core::Encodable;

    use super::Msx;
    use crate::core::identifiers::RequestId;

    #[test]
    fn ensure_last_byte_zero_always() {
        let data = Msx {
            reqi: RequestId(1),
            msg: "aaaaaa".into(),
        };

        let mut buf = BytesMut::new();
        let res = data.encode(&mut buf, None);
        assert!(res.is_ok());

        assert_eq!(buf.last(), Some(&0));
        assert_eq!(buf.len(), 98);
    }
}
