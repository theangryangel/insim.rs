use crate::proto;

use std::io;
use bytes::{Bytes, BytesMut};
use deku::{DekuContainerRead, DekuContainerWrite};
use tokio_util::codec::{Decoder, Encoder, LengthDelimitedCodec};
use std::convert::TryFrom;

pub struct InsimCodec {
    inner: LengthDelimitedCodec,
}

impl InsimCodec {
    pub fn new() -> InsimCodec {
        let codec = LengthDelimitedCodec::builder()
            .length_field_length(1)
            .length_adjustment(-1)
            .big_endian()
            .new_codec();
    
        InsimCodec{inner: codec}
    }
}

impl Decoder for InsimCodec {
    type Item = proto::Insim;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> io::Result<Option<proto::Insim>> {
        let data = self.inner
            .decode(src);

        match data {
            Ok(None) => Ok(None),
            Ok(Some(data)) => {
                let res = proto::Insim::try_from(data.as_ref());

                match res {
                    Ok(packet) => Ok(Some(packet)),
                    Err(e) => {
                        Err(
                            io::Error::new(
                                io::ErrorKind::InvalidInput,
                                e.to_string(),
                            )
                        )
                    }
                }
            },
            Err(e) => Err(e)
        }
    }
}

impl Encoder<proto::Insim> for InsimCodec {
    type Error = io::Error;

    fn encode(&mut self, msg: proto::Insim, dst: &mut BytesMut) -> Result<(), io::Error> {
        self.inner.encode(
            Bytes::from(msg.to_bytes().unwrap()),
            dst,
        )
    }
}
