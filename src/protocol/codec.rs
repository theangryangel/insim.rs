use super::Packet;
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
    type Item = Packet;

    // TODO return custom error
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> io::Result<Option<Self::Item>> {
        if src.is_empty() {
            return Ok(None);
        }

        let data = self.inner.decode(src);

        match data {
            Err(e) => Err(e),
            Ok(None) => Ok(None),
            Ok(Some(data)) => {
                let res = Self::Item::try_from(data.as_ref());

                match res {
                    Ok(packet) => Ok(Some(packet)),
                    Err(DekuError::Incomplete(e)) => {
                        // If we're here, everything is going wonky.
                        // We could just discard the packet and move on, but thats probably a bad
                        // thing?
                        tracing::error!(
                            "malformed packet! This is probably a programming error. Error: {:?}, Input: {:?}",
                            e,
                            data.to_vec(),
                        );
                        Err(io::Error::new(
                            io::ErrorKind::InvalidInput,
                            "incomplete or malformed packet",
                        ))
                    }
                    Err(DekuError::Parse(e)) => {
                        tracing::warn!("unsupported packet: {:?}", e);
                        Ok(None)
                    }
                    Err(e) => Err(io::Error::new(io::ErrorKind::InvalidInput, e.to_string())),
                }
            }
        }
    }
}

impl Encoder<Packet> for InsimCodec {
    type Error = io::Error;

    fn encode(&mut self, msg: Packet, dst: &mut BytesMut) -> Result<(), io::Error> {
        self.inner.encode(Bytes::from(msg.to_bytes().unwrap()), dst)
    }
}
