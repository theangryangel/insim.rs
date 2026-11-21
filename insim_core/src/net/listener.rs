use super::{DatagramSocket, Decoder, ReceiveTransport, Result, Runtime, Socket, Transport};

/// Standalone UDP packet receiver, also consumable by an InSim session's `attach`.
/// Listeners have no inactivity timeout.
#[derive(Debug)]
pub struct Listener<R: Runtime, D: Decoder> {
    transport: Transport<R::Udp, D>,
}
impl<R: Runtime, D: Decoder> Listener<R, D> {
    /// Construct a UDP listener with a decoder or a `Demux` stack.
    pub fn udp<S: DatagramSocket<Runtime = R>>(socket: S, decoder: D) -> Result<Self> {
        Ok(Self {
            transport: Transport::new(socket.into_udp(), decoder)?,
        })
    }
    /// Try to receive one packet without waiting.
    pub fn try_read(&mut self) -> Result<Option<D::Item>> {
        self.transport.try_read()
    }
    /// The underlying socket, for address inspection.
    pub fn socket(&self) -> &R::Udp {
        self.transport.socket()
    }
}
impl<R: Runtime, D: Decoder> ReceiveTransport for Listener<R, D> {
    type Runtime = R;
    type Item = D::Item;
    fn try_read(&mut self) -> Result<Option<D::Item>> {
        self.try_read()
    }
    fn socket(&self) -> &dyn Socket<Runtime = R> {
        self.transport.socket()
    }
}
#[cfg(feature = "blocking")]
impl<D: Decoder> Listener<super::Blocking, D> {
    /// Wait for the next packet. A malformed datagram remainder is reported and discarded.
    pub fn read(&mut self) -> Result<D::Item> {
        loop {
            if let Some(packet) = self.try_read()? {
                return Ok(packet);
            }
            super::Blocking::wait(&[(self.transport.socket(), super::Interest::READ)], None)?;
        }
    }
}
#[cfg(feature = "tokio")]
impl<D: Decoder> Listener<super::Tokio, D> {
    /// Wait for the next packet. Cancellation preserves all buffered progress.
    pub async fn read(&mut self) -> Result<D::Item> {
        loop {
            if let Some(packet) = self.try_read()? {
                return Ok(packet);
            }
            super::Tokio::wait(&[(self.transport.socket(), super::Interest::READ)], None).await?;
        }
    }
}
