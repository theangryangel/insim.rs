use bytes::{Buf, BufMut, BytesMut};
use insim_core::{
    Decode, DecodeContext, Encode, EncodeContext, EncodeErrorKind,
    net::{Decoder, Encoder, Error, Result},
};

use crate::{MAX_SIZE_PACKET, Packet};

/// Pure InSim packet framing and translation. No clocks, buffers or protocol state.
#[derive(Debug, Default)]
pub struct InsimCodec;
impl Decoder for InsimCodec {
    type Item = Packet;
    fn accepts(&self, input: &[u8]) -> bool {
        // InSim has no universal magic number. Check framing and the known type range;
        // ambiguous telemetry formats require explicit decoder ordering.
        input.len() >= 4
            && input[0] != 0
            && (cfg!(feature = "allow-unknown-packet") || (1..=70).contains(&input[1]))
            && input[0] as usize * 4 <= input.len()
    }
    fn decode(&mut self, input: &mut BytesMut) -> Result<Option<Packet>> {
        if input.is_empty() {
            return Ok(None);
        }
        let n = input[0] as usize * 4;
        if n < 4 {
            return Err(Error::Frame("invalid InSim packet length"));
        }
        if input.len() < n {
            return Ok(None);
        }
        let original = input.split_to(n).freeze();
        let mut bytes = original.clone();
        bytes.advance(1);
        let packet = match Packet::decode(&mut DecodeContext::new(&mut bytes)) {
            Ok(packet) => packet,
            Err(error) => {
                return Err(Error::Decode {
                    offset: original.len() - bytes.len(),
                    input: original,
                    error,
                });
            },
        };
        if !bytes.is_empty() {
            return Err(Error::IncompleteDecode {
                input: original,
                remaining: bytes,
            });
        }
        tracing::trace!(?packet, "decoded InSim frame");
        Ok(Some(packet))
    }
}
impl Encoder for InsimCodec {
    fn encode(&self, packet: &Packet, output: &mut BytesMut) -> Result<()> {
        let mut frame = BytesMut::with_capacity(packet.size_hint());
        frame.put_u8(0);
        packet.encode(&mut EncodeContext::new(&mut frame))?;
        if !(4..=MAX_SIZE_PACKET).contains(&frame.len()) || !frame.len().is_multiple_of(4) {
            return Err(EncodeErrorKind::OutOfRange {
                min: 4,
                max: MAX_SIZE_PACKET,
                found: frame.len(),
            }
            .context("InSim length must be a multiple of four")
            .into());
        }
        frame[0] = (frame.len() / 4) as u8;
        output.extend_from_slice(&frame);
        Ok(())
    }
    fn frame_len(&self, output: &[u8]) -> Result<usize> {
        let n = output.first().copied().unwrap_or(0) as usize * 4;
        if n < 4 || n > output.len() {
            return Err(Error::Frame("invalid queued InSim frame"));
        }
        Ok(n)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        WithRequestId,
        identifiers::RequestId,
        insim::{TinyType, Ver},
    };
    #[test]
    fn incremental_decoding_and_exact_encoding() {
        let mut codec = InsimCodec;
        let mut input = BytesMut::from(&[1, 3, 2][..]);
        assert!(codec.decode(&mut input).unwrap().is_none());
        assert_eq!(input.as_ref(), &[1, 3, 2]);
        input.extend_from_slice(&[3, 1, 3, 4, 3]);
        for reqi in [2, 4] {
            let packet = codec.decode(&mut input).unwrap().unwrap();
            let mut encoded = BytesMut::new();
            codec.encode(&packet, &mut encoded).unwrap();
            assert_eq!(encoded.as_ref(), &[1, 3, reqi, 3]);
        }
        assert!(input.is_empty());
    }
    #[test]
    fn version_compatibility_is_not_a_decoding_policy() {
        let mut codec = InsimCodec;
        let mut input = BytesMut::new();
        codec
            .encode(
                &Ver {
                    insimver: 1,
                    ..Ver::default()
                }
                .into(),
                &mut input,
            )
            .unwrap();
        assert!(matches!(
            codec.decode(&mut input).unwrap(),
            Some(Packet::Ver(Ver { insimver: 1, .. }))
        ));
    }
    #[test]
    fn queued_output_survives_encode_failure() {
        let codec = InsimCodec;
        let mut output = BytesMut::new();
        codec
            .encode(
                &TinyType::Ping.with_request_id(RequestId(2)).into(),
                &mut output,
            )
            .unwrap();
        let original = output.clone();
        assert!(
            codec
                .encode(&Packet::Outsim2(Default::default()), &mut output)
                .is_err()
        );
        assert_eq!(output, original);
    }
}
