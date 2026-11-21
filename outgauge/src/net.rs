//! Standalone or attachable OutGauge UDP reception.
use bytes::BytesMut;
use insim_core::{
    Decode, DecodeContext,
    net::{self, DatagramSocket, Decoder, ReceiveTransport, Runtime, Socket},
};

use crate::Outgauge;

/// Pure OutGauge datagram decoder. Recognizes the 92- and 96-byte wire formats.
#[derive(Debug, Default)]
pub struct OutGaugeCodec;
impl Decoder for OutGaugeCodec {
    type Item = Outgauge;
    fn accepts(&self, input: &[u8]) -> bool {
        matches!(input.len(), 92 | 96)
    }
    fn decode(&mut self, input: &mut BytesMut) -> net::Result<Option<Outgauge>> {
        if !self.accepts(input) {
            return Err(net::Error::Frame("invalid OutGauge datagram length"));
        }
        let original = input.split().freeze();
        let mut bytes = original.clone();
        let packet = Outgauge::decode(&mut DecodeContext::new(&mut bytes)).map_err(|error| {
            net::Error::Decode {
                offset: original.len() - bytes.len(),
                input: original,
                error,
            }
        })?;
        if !bytes.is_empty() {
            return Err(net::Error::Frame("trailing OutGauge bytes"));
        }
        Ok(Some(packet))
    }
}
/// An OutGauge receiver usable standalone or with `Session::attach`.
#[derive(Debug)]
pub struct Listener<R: Runtime>(net::Listener<R, OutGaugeCodec>);
impl<R: Runtime> Listener<R> {
    /// Take ownership of a UDP socket; its type selects the runtime.
    pub fn udp<S: DatagramSocket<Runtime = R>>(socket: S) -> net::Result<Self> {
        Ok(Self(net::Listener::udp(socket, OutGaugeCodec)?))
    }
    /// Receive without waiting.
    pub fn try_read(&mut self) -> net::Result<Option<Outgauge>> {
        self.0.try_read()
    }
    /// Inspect the socket and its bound address.
    pub fn socket(&self) -> &R::Udp {
        self.0.socket()
    }
}
impl<R: Runtime> ReceiveTransport for Listener<R> {
    type Runtime = R;
    type Item = Outgauge;
    fn try_read(&mut self) -> net::Result<Option<Outgauge>> {
        self.try_read()
    }
    fn socket(&self) -> &dyn Socket<Runtime = R> {
        self.0.socket()
    }
}
#[cfg(feature = "blocking")]
impl Listener<net::Blocking> {
    /// Wait for the next OutGauge datagram.
    pub fn read(&mut self) -> net::Result<Outgauge> {
        self.0.read()
    }
}
#[cfg(feature = "tokio")]
impl Listener<net::Tokio> {
    /// Wait for the next OutGauge datagram, preserving progress on cancellation.
    pub async fn read(&mut self) -> net::Result<Outgauge> {
        self.0.read().await
    }
}
