use crate::identifiers::RequestId;

#[derive(Debug, Clone, Default, insim_core::Decode, insim_core::Encode)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
/// Extended Message (like [Mst](super::Mst), but longer)
pub struct Msx {
    /// Non-zero if the packet is a packet request or a reply to a request
    #[insim(pad_after = 1)]
    pub reqi: RequestId,

    /// Message
    #[insim(codepage(length = 96, trailing_nul = true))]
    pub msg: String,
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use bytes::{BufMut, BytesMut};
    use insim_core::Encode;

    use super::*;

    #[test]
    fn test_msx() {
        let mut data = BytesMut::new();
        data.extend_from_slice(&[
            1, // reqi
            0,
        ]);

        data.extend_from_slice(b"aaaaaa");
        data.put_bytes(0, 96 - 6);

        assert_from_to_bytes!(Msx, data.as_ref(), |msx: Msx| {
            assert_eq!(&msx.msg, "aaaaaa");
        });
    }

    #[test]
    fn test_contextual_error() {
        let msx = Msx {
            msg: ("xK9#mZ2$vL!pQ@nR&wJ*yT(hB)cF+dA-eG=sU/iO\\uX{jY}lN|oP~qS
                xK9#mZ2$vL!pQ@nR&wJ*yT(hB)cF+dA-eG=sU/iO\\uX{jY}lN|oP~qS")
                .to_string(),
            ..Default::default()
        };

        let mut buf = BytesMut::new();
        let res = msx.encode(&mut buf);

        assert!(res.is_err());
        assert!(matches!(
            res,
            Err(insim_core::EncodeError {
                context: Some(Cow::Borrowed("Msx::msg")),
                kind: insim_core::EncodeErrorKind::Nested { .. }
            })
        ));
    }
}
