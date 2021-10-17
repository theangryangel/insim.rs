use crate::packets;
use bytes::{Bytes, BytesMut};
use deku::{DekuContainerWrite, DekuError};
use std::convert::TryFrom;
use std::io;
use tokio_util::codec::{Decoder, Encoder, LengthDelimitedCodec};
use tracing;

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

        InsimCodec { inner: codec }
    }
}

impl Default for InsimCodec {
    fn default() -> Self {
        Self::new()
    }
}

impl Decoder for InsimCodec {
    type Item = packets::Insim;

    // TODO return custom error
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> io::Result<Option<packets::Insim>> {
        if src.is_empty() {
            return Ok(None);
        }

        let data = self.inner.decode(src);

        match data {
            Err(e) => Err(e),
            Ok(None) => Ok(None),
            Ok(Some(data)) => {
                let res = packets::Insim::try_from(data.as_ref());

                match res {
                    Ok(packet) => Ok(Some(packet)),
                    Err(DekuError::Incomplete(e)) => {
                        // If we're here, everything is going wonky.
                        // We could just discard the packet and move on, but thats probably a bad
                        // thing?
                        panic!(
                            "Malformed packet! This is probably a programming error. Error: {:?}, Input: {:?}",
                            e,
                            data.to_vec(),
                        )
                    }
                    Err(DekuError::Parse(e)) => {
                        tracing::debug!("Unsupported packet {:?} {:?}", e, e.to_string());
                        Ok(None)
                    }
                    Err(e) => Err(io::Error::new(io::ErrorKind::InvalidInput, e.to_string())),
                }
            }
        }
    }
}

impl Encoder<packets::Insim> for InsimCodec {
    type Error = io::Error;

    fn encode(&mut self, msg: packets::Insim, dst: &mut BytesMut) -> Result<(), io::Error> {
        self.inner.encode(Bytes::from(msg.to_bytes().unwrap()), dst)
    }
}
