//! Standalone or attachable OutSim UDP reception.
use bytes::BytesMut;
use insim_core::{
    Decode, DecodeContext,
    net::{self, DatagramSocket, Decoder, ReceiveTransport, Runtime, Socket},
};

use crate::{
    OutsimPack,
    outsimpack2::{OutSimOpts, OutsimPack2},
};

/// Pure OutSim decoder with a packet type selected at construction.
///
/// `OutSimCodec::default()` decodes [`OutsimPack`].
/// `OutSimCodec::extended(opts)` decodes [`OutsimPack2`] using the sender's
/// configured OutSim Opts. The formats are never automatically interchanged.
#[derive(Debug)]
pub struct OutSimCodec<P = OutsimPack> {
    opts: OutSimOpts,
    packet: std::marker::PhantomData<fn() -> P>,
}
impl Default for OutSimCodec {
    fn default() -> Self {
        Self {
            opts: OutSimOpts::empty(),
            packet: std::marker::PhantomData,
        }
    }
}
impl OutSimCodec {
    /// Select the extended packet format and its configured fields.
    /// The options must match LFS's OutSim Opts setting.
    pub fn extended(opts: OutSimOpts) -> OutSimCodec<OutsimPack2> {
        OutSimCodec {
            opts,
            packet: std::marker::PhantomData,
        }
    }
}
impl Decoder for OutSimCodec {
    type Item = OutsimPack;
    fn accepts(&self, input: &[u8]) -> bool {
        matches!(input.len(), 64 | 68)
    }
    fn decode(&mut self, input: &mut BytesMut) -> net::Result<Option<OutsimPack>> {
        if !self.accepts(input) {
            return Err(net::Error::Frame("invalid original OutSim length"));
        }
        decode(input, OutsimPack::decode).map(Some)
    }
}
impl OutSimCodec<OutsimPack2> {
    fn extended_len(&self) -> usize {
        // Field widths on the wire, including each section's padding.
        [
            (OutSimOpts::HEADER, 4),
            (OutSimOpts::ID, 4),
            (OutSimOpts::TIME, 4),
            (OutSimOpts::MAIN, 60),
            (OutSimOpts::INPUTS, 20),
            (OutSimOpts::DRIVE, 12),
            (OutSimOpts::DISTANCE, 8),
            (OutSimOpts::WHEELS, 160),
            (OutSimOpts::EXTRA, 8),
        ]
        .into_iter()
        .filter(|(flag, _)| self.opts.contains(*flag))
        .map(|(_, size)| size)
        .sum()
    }
}
impl Decoder for OutSimCodec<OutsimPack2> {
    type Item = OutsimPack2;
    fn accepts(&self, input: &[u8]) -> bool {
        if self.opts.contains(OutSimOpts::HEADER) {
            input.starts_with(b"OSIM")
        } else {
            !input.is_empty() && input.len() == self.extended_len()
        }
    }
    fn decode(&mut self, input: &mut BytesMut) -> net::Result<Option<OutsimPack2>> {
        if input.is_empty() || input.len() != self.extended_len() {
            return Err(net::Error::Frame("invalid extended OutSim length"));
        }
        if !self.accepts(input) {
            return Err(net::Error::Frame("invalid extended OutSim header"));
        }
        decode(input, |ctx| {
            OutsimPack2::decode_with_options(ctx, &self.opts)
        })
        .map(Some)
    }
}

fn decode<T>(
    input: &mut BytesMut,
    f: impl FnOnce(&mut DecodeContext<'_>) -> Result<T, insim_core::DecodeError>,
) -> net::Result<T> {
    let original = input.split().freeze();
    let mut bytes = original.clone();
    let packet = f(&mut DecodeContext::new(&mut bytes)).map_err(|error| net::Error::Decode {
        offset: original.len() - bytes.len(),
        input: original,
        error,
    })?;
    if !bytes.is_empty() {
        return Err(net::Error::Frame("trailing OutSim bytes"));
    }
    Ok(packet)
}
/// An OutSim receiver with a statically selected packet format.
/// Usable standalone or with `Session::attach`; it has no inactivity timeout.
#[derive(Debug)]
pub struct Listener<R: Runtime, D: Decoder = OutSimCodec>(net::Listener<R, D>);
impl<R: Runtime> Listener<R> {
    /// Receive original OutSim packets from an owned UDP socket.
    pub fn udp<S: DatagramSocket<Runtime = R>>(socket: S) -> net::Result<Self> {
        Ok(Self(net::Listener::udp(socket, OutSimCodec::default())?))
    }
    /// Receive extended OutSim packets using the sender's configured OutSim Opts.
    pub fn extended<S: DatagramSocket<Runtime = R>>(
        socket: S,
        opts: OutSimOpts,
    ) -> net::Result<Listener<R, OutSimCodec<OutsimPack2>>> {
        Ok(Listener(net::Listener::udp(
            socket,
            OutSimCodec::extended(opts),
        )?))
    }
}
impl<R: Runtime, D: Decoder> Listener<R, D> {
    /// Receive the selected packet type without waiting.
    pub fn try_read(&mut self) -> net::Result<Option<D::Item>> {
        self.0.try_read()
    }
    /// Inspect the socket and its bound address.
    pub fn socket(&self) -> &R::Udp {
        self.0.socket()
    }
}
impl<R: Runtime, D: Decoder> ReceiveTransport for Listener<R, D> {
    type Runtime = R;
    type Item = D::Item;
    fn try_read(&mut self) -> net::Result<Option<D::Item>> {
        self.try_read()
    }
    fn socket(&self) -> &dyn Socket<Runtime = R> {
        self.0.socket()
    }
}
#[cfg(feature = "blocking")]
impl<D: Decoder> Listener<net::Blocking, D> {
    /// Wait for the next packet of the selected format.
    pub fn read(&mut self) -> net::Result<D::Item> {
        self.0.read()
    }
}
#[cfg(feature = "tokio")]
impl<D: Decoder> Listener<net::Tokio, D> {
    /// Wait for the next packet of the selected format, preserving progress on cancellation.
    pub async fn read(&mut self) -> net::Result<D::Item> {
        self.0.read().await
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use insim_core::Encode;

    use super::*;

    fn original(with_id: bool) -> BytesMut {
        let mut packet = OutsimPack::decode_slice([0; 64]).unwrap();
        packet.time = Duration::from_millis(1234);
        packet.id = with_id.then_some(crate::OutsimId(42));
        BytesMut::from(packet.to_bytes().unwrap().as_ref())
    }
    fn extended() -> OutsimPack2 {
        // Full wire layout: header + ID + time + motion + inputs + drivetrain +
        // distance + four wheels + extra, including protocol padding.
        let mut bytes = vec![0; 280];
        bytes[..4].copy_from_slice(b"OSIM");
        let mut bytes = bytes::Bytes::from(bytes);
        OutsimPack2::decode_with_options(&mut DecodeContext::new(&mut bytes), &OutSimOpts::all())
            .unwrap()
    }
    fn encode_extended(packet: &OutsimPack2, opts: OutSimOpts) -> BytesMut {
        let mut output = BytesMut::new();
        packet
            .encode_with_options(&mut insim_core::EncodeContext::new(&mut output), &opts)
            .unwrap();
        output
    }
    #[test]
    fn default_codec_accepts_original_packets_with_and_without_id() {
        let mut codec = OutSimCodec::default();
        for with_id in [false, true] {
            let mut input = original(with_id);
            assert!(codec.accepts(&input));
            let packet: OutsimPack = codec.decode(&mut input).unwrap().unwrap();
            assert_eq!(packet.time, Duration::from_millis(1234));
            assert_eq!(packet.id, with_id.then_some(crate::OutsimId(42)));
            assert!(input.is_empty());
        }
    }
    #[test]
    fn all_extended_option_combinations_match_the_packet_encoder() {
        let packet = extended();
        for bits in 1..=OutSimOpts::all().bits() {
            let opts = OutSimOpts::from_bits_retain(bits);
            let mut codec = OutSimCodec::extended(opts);
            let mut input = encode_extended(&packet, opts);
            assert_eq!(codec.extended_len(), input.len(), "options {opts:?}");
            assert!(codec.accepts(&input));
            let decoded: OutsimPack2 = codec.decode(&mut input).unwrap().unwrap();
            assert_eq!(decoded.header.is_some(), opts.contains(OutSimOpts::HEADER));
            assert!(input.is_empty());
        }
    }
    #[test]
    fn malformed_claimed_extended_packet_does_not_fall_back_to_original() {
        let mut codec = OutSimCodec::extended(OutSimOpts::all());
        let mut input = original(false);
        input[..4].copy_from_slice(b"OSIM");
        assert!(codec.accepts(&input));
        assert!(codec.decode(&mut input).is_err());
        // Format selection is fixed: original packets are not accepted as a fallback.
        assert!(!codec.accepts(&original(false)));
        assert!(codec.decode(&mut original(false)).is_err());
        let mut valid = encode_extended(&extended(), OutSimOpts::all());
        let _: OutsimPack2 = codec.decode(&mut valid).unwrap().unwrap();
    }
    #[test]
    fn configured_headerless_layout_has_a_fixed_extended_output_type() {
        let opts = OutSimOpts::ID | OutSimOpts::MAIN;
        let mut codec = OutSimCodec::extended(opts);
        let mut packet = extended();
        packet.id = Some(crate::OutsimId(7));
        let mut input = encode_extended(&packet, opts);
        assert_eq!(input.len(), 64);
        let packet: OutsimPack2 = codec.decode(&mut input).unwrap().unwrap();
        assert_eq!(packet.id, Some(crate::OutsimId(7)));
        assert!(packet.time.is_none());
    }
    #[test]
    fn unrecognized_lengths_are_not_claimed() {
        assert!(!OutSimCodec::default().accepts(&[0; 92]));
        assert!(!OutSimCodec::extended(OutSimOpts::ID).accepts(&[0; 92]));
    }
}
