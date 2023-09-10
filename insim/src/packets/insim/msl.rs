use insim_core::{identifiers::RequestId, prelude::*, string::codepages, EncodableError};

#[cfg(feature = "serde")]
use serde::Serialize;

/// Enum for the sound field of [Msl].
#[derive(Debug, Default, InsimEncode, InsimDecode, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[repr(u8)]
pub enum SoundType {
    #[default]
    Silent = 0,

    Message = 1,

    SystemMessage = 2,

    InvalidKey = 3,

    // This is referred to as "Error" in the Insim documentation, but this is a special word in
    // rust so I'm trying to avoid it.
    Failure = 4,
}

#[derive(Debug, InsimDecode, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
/// Send a message to the local computer only. If you are connected to a server this means the
/// console. If you are connected to a client this means to the local client only.
pub struct Msl {
    pub reqi: RequestId,

    pub sound: SoundType,

    #[insim(bytes = "128")]
    pub msg: String,
}

impl Encodable for Msl {
    fn encode(
        &self,
        buf: &mut bytes::BytesMut,
        limit: Option<insim_core::ser::Limit>,
    ) -> Result<(), insim_core::EncodableError> {
        if limit.is_some() {
            return Err(EncodableError::UnexpectedLimit(format!(
                "Msl does not support limit! {limit:?}",
            )));
        }

        self.reqi.encode(buf, None)?;
        self.sound.encode(buf, None)?;

        let msg: &[u8] = &codepages::to_lossy_bytes(&self.msg);
        if msg.len() > 127 {
            return Err(EncodableError::WrongSize(
                "Msx only supports upto 127 byte long messages".into(),
            ));
        }

        // last byte must be zero
        let padding = 128 - msg.len();
        buf.extend_from_slice(msg);
        if padding > 0 {
            buf.put_bytes(0, padding);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use bytes::BytesMut;
    use insim_core::Encodable;

    use super::{Msl, SoundType};
    use crate::core::identifiers::RequestId;

    #[test]
    fn ensure_last_byte_zero_always() {
        let data = Msl {
            reqi: RequestId(1),
            sound: SoundType::Silent,
            msg: "aaaaaa".into(),
        };

        let mut buf = BytesMut::new();
        let res = data.encode(&mut buf, None);
        assert!(res.is_ok());

        assert_eq!(buf.last(), Some(&0));
        assert_eq!(buf.len(), 130);
    }
}
