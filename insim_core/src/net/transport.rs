use std::{fmt::Debug, io};

use bytes::{Buf, BytesMut};

use super::{Decoder, Encoder, Error, Result, Runtime, Socket};

/// A receiving transport, independent of its concrete socket and decoder types.
pub trait ReceiveTransport: Debug + Send {
    /// Runtime shared by all sockets attached to a session.
    type Runtime: Runtime;
    /// Decoded packet.
    type Item: Debug + Send + 'static;
    /// Make bounded nonblocking progress and return a packet if available.
    fn try_read(&mut self) -> Result<Option<Self::Item>>;
    /// Socket used for readiness waiting.
    fn socket(&self) -> &dyn Socket<Runtime = Self::Runtime>;
}

/// One socket, its decoder, and persistent input/output progress.
///
/// TCP accumulates partial input. UDP drains complete packets from the current
/// datagram before receiving another one. A malformed remainder is reported and
/// discarded without losing previously returned packets or joining datagrams.
#[derive(Debug)]
pub struct Transport<S: Socket, D: Decoder> {
    socket: S,
    decoder: D,
    incoming: BytesMut,
    outgoing: BytesMut,
    scratch: Box<[u8]>,
    failed: bool,
}
impl<S: Socket, D: Decoder> Transport<S, D> {
    /// Take ownership of a socket and switch it to nonblocking operation.
    pub fn new(socket: S, decoder: D) -> Result<Self> {
        socket.prepare()?;
        let capacity = if socket.is_datagram() { 65536 } else { 4096 };
        Ok(Self {
            socket,
            decoder,
            incoming: BytesMut::new(),
            outgoing: BytesMut::new(),
            scratch: vec![0; capacity].into_boxed_slice(),
            failed: false,
        })
    }
    /// Number of encoded bytes still awaiting socket acceptance.
    pub fn pending_bytes(&self) -> usize {
        self.outgoing.len()
    }
    /// The socket, for address inspection and readiness waiting.
    pub fn socket(&self) -> &S {
        &self.socket
    }
    fn read_inner(&mut self) -> Result<Option<D::Item>> {
        if !self.incoming.is_empty()
            && let Some(packet) = self.decode_packet()?
        {
            return Ok(Some(packet));
        }
        let n = match self.socket.try_recv(&mut self.scratch) {
            Ok(n) => n,
            Err(e)
                if matches!(
                    e.kind(),
                    io::ErrorKind::WouldBlock | io::ErrorKind::Interrupted
                ) =>
            {
                return Ok(None);
            },
            Err(e) => return Err(e.into()),
        };
        if n == 0 {
            return Err(if self.socket.is_datagram() {
                Error::Frame("empty datagram")
            } else {
                Error::Disconnected
            });
        }
        // UDP reaches this point only after the previous datagram is drained.
        self.incoming.extend_from_slice(&self.scratch[..n]);
        if self.socket.is_datagram() && !self.decoder.accepts(&self.incoming) {
            self.incoming.clear();
            return Err(Error::Frame("unrecognized datagram"));
        }
        self.decode_packet()
    }
    fn decode_packet(&mut self) -> Result<Option<D::Item>> {
        let before = self.incoming.len();
        let result = match self.decoder.decode(&mut self.incoming) {
            Ok(Some(_)) if self.incoming.len() >= before => {
                Err(Error::Frame("decoder made no progress"))
            },
            Ok(None) if self.socket.is_datagram() => {
                Err(Error::Frame("incomplete packet in datagram"))
            },
            result => result,
        };
        // An incomplete decode consumed exactly one frame; the rest is still valid.
        if result.is_err()
            && self.socket.is_datagram()
            && !matches!(result, Err(Error::IncompleteDecode { .. }))
        {
            // A later receive must never complete a packet from this datagram.
            self.incoming.clear();
        }
        result
    }
}
impl<S: Socket, D: Decoder> ReceiveTransport for Transport<S, D> {
    type Runtime = S::Runtime;
    type Item = D::Item;
    fn try_read(&mut self) -> Result<Option<D::Item>> {
        if self.failed {
            return Err(Error::Disconnected);
        }
        let result = self.read_inner();
        // An incomplete decode leaves the stream framed, so it is not terminal.
        if result.is_err()
            && !self.socket.is_datagram()
            && !matches!(result, Err(Error::IncompleteDecode { .. }))
        {
            self.failed = true;
        }
        result
    }
    fn socket(&self) -> &dyn Socket<Runtime = S::Runtime> {
        &self.socket
    }
}
impl<S: Socket, D: Encoder> Transport<S, D> {
    /// Encode a packet into persistent output storage before any waiting occurs.
    pub fn queue(&mut self, packet: &D::Item) -> Result<()> {
        if self.failed {
            return Err(Error::Disconnected);
        }
        self.decoder.encode(packet, &mut self.outgoing)
    }
    /// Attempt bounded write progress. Returns whether all queued output was accepted.
    pub fn try_flush(&mut self) -> Result<bool> {
        if self.failed {
            return Err(Error::Disconnected);
        }
        let result = self.flush_inner();
        if result.is_err() {
            self.failed = true;
        }
        result
    }
    fn flush_inner(&mut self) -> Result<bool> {
        // A budget prevents a producer or a writable socket from monopolizing a pump pass.
        for _ in 0..32 {
            if self.outgoing.is_empty() {
                return Ok(true);
            }
            let n = if self.socket.is_datagram() {
                self.decoder.frame_len(&self.outgoing)?
            } else {
                self.outgoing.len()
            };
            if n == 0 || n > self.outgoing.len() {
                return Err(Error::Frame("invalid outgoing frame length"));
            }
            match self.socket.try_send(&self.outgoing[..n]) {
                Ok(0) => {
                    return Err(io::Error::new(
                        io::ErrorKind::WriteZero,
                        "socket accepted no bytes",
                    )
                    .into());
                },
                Ok(sent) if self.socket.is_datagram() && sent != n => {
                    return Err(
                        io::Error::new(io::ErrorKind::WriteZero, "short datagram send").into(),
                    );
                },
                Ok(sent) => self.outgoing.advance(sent),
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => return Ok(false),
                Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
                Err(e) => return Err(e.into()),
            }
        }
        Ok(self.outgoing.is_empty())
    }
}

#[cfg(test)]
mod tests {
    use std::{
        collections::VecDeque,
        net::SocketAddr,
        sync::{Arc, Mutex},
    };

    use bytes::Bytes;

    use super::*;
    use crate::net::{DatagramSocket, StreamSocket};

    #[derive(Debug)]
    struct TestRuntime;
    impl Runtime for TestRuntime {
        type Tcp = FakeSocket;
        type Udp = FakeSocket;
    }
    #[derive(Debug, Default)]
    struct Script {
        reads: VecDeque<Vec<u8>>,
        writes: VecDeque<io::Result<usize>>,
        sent: Vec<Vec<u8>>,
    }
    #[derive(Debug, Clone)]
    struct FakeSocket {
        datagram: bool,
        script: Arc<Mutex<Script>>,
    }
    impl Socket for FakeSocket {
        type Runtime = TestRuntime;
        fn is_datagram(&self) -> bool {
            self.datagram
        }
        fn prepare(&self) -> io::Result<()> {
            Ok(())
        }
        fn try_recv(&self, buf: &mut [u8]) -> io::Result<usize> {
            let mut script = self.script.lock().unwrap();
            let data = script.reads.pop_front().ok_or(io::ErrorKind::WouldBlock)?;
            buf[..data.len()].copy_from_slice(&data);
            Ok(data.len())
        }
        fn try_send(&self, buf: &[u8]) -> io::Result<usize> {
            let mut script = self.script.lock().unwrap();
            let n = script.writes.pop_front().unwrap_or(Ok(buf.len()))?;
            script.sent.push(buf[..n].to_vec());
            Ok(n)
        }
        fn local_addr(&self) -> io::Result<SocketAddr> {
            Ok("127.0.0.1:1".parse().unwrap())
        }
        fn shutdown(&self) -> io::Result<()> {
            Ok(())
        }
    }
    impl StreamSocket for FakeSocket {
        fn into_tcp(self) -> FakeSocket {
            self
        }
    }
    impl DatagramSocket for FakeSocket {
        fn into_udp(self) -> FakeSocket {
            self
        }
    }
    #[derive(Debug)]
    struct Frames;
    impl Decoder for Frames {
        type Item = Bytes;
        fn accepts(&self, _: &[u8]) -> bool {
            true
        }
        fn decode(&mut self, input: &mut BytesMut) -> Result<Option<Bytes>> {
            let Some(&n) = input.first() else {
                return Ok(None);
            };
            if n == 0 {
                return Err(Error::Frame("zero length"));
            }
            if input.len() < n as usize {
                return Ok(None);
            }
            Ok(Some(input.split_to(n as usize).freeze()))
        }
    }
    impl Encoder for Frames {
        fn encode(&self, packet: &Bytes, output: &mut BytesMut) -> Result<()> {
            output.extend_from_slice(packet);
            Ok(())
        }
        fn frame_len(&self, output: &[u8]) -> Result<usize> {
            Ok(output[0] as usize)
        }
    }
    fn transport(datagram: bool) -> (Transport<FakeSocket, Frames>, Arc<Mutex<Script>>) {
        let script = Arc::new(Mutex::new(Script::default()));
        (
            Transport::new(
                FakeSocket {
                    datagram,
                    script: script.clone(),
                },
                Frames,
            )
            .unwrap(),
            script,
        )
    }
    #[test]
    fn tcp_partial_writes_survive_would_block_and_more_queueing() {
        let (mut transport, script) = transport(false);
        script
            .lock()
            .unwrap()
            .writes
            .extend([Ok(2), Err(io::ErrorKind::WouldBlock.into())]);
        transport.queue(&Bytes::from_static(&[4, 1, 2, 3])).unwrap();
        assert!(!transport.try_flush().unwrap());
        assert_eq!(transport.pending_bytes(), 2);
        transport.queue(&Bytes::from_static(&[2, 4])).unwrap();
        assert!(transport.try_flush().unwrap());
        assert_eq!(script.lock().unwrap().sent.concat(), [4, 1, 2, 3, 2, 4]);
    }
    #[test]
    fn udp_preserves_datagrams_across_would_block() {
        let (mut transport, script) = transport(true);
        script
            .lock()
            .unwrap()
            .writes
            .push_back(Err(io::ErrorKind::WouldBlock.into()));
        transport.queue(&Bytes::from_static(&[3, 1, 2])).unwrap();
        transport.queue(&Bytes::from_static(&[2, 3])).unwrap();
        assert!(!transport.try_flush().unwrap());
        assert_eq!(transport.pending_bytes(), 5);
        assert!(transport.try_flush().unwrap());
        assert_eq!(script.lock().unwrap().sent, [vec![3, 1, 2], vec![2, 3]]);
    }
    #[test]
    fn zero_write_and_short_udp_send_are_terminal() {
        for (datagram, accepted) in [(false, 0), (true, 1)] {
            let (mut transport, script) = transport(datagram);
            script.lock().unwrap().writes.push_back(Ok(accepted));
            transport.queue(&Bytes::from_static(&[2, 1])).unwrap();
            assert!(transport.try_flush().is_err());
            assert_eq!(transport.pending_bytes(), 2);
            assert!(matches!(transport.try_flush(), Err(Error::Disconnected)));
        }
    }
    #[test]
    fn tcp_retains_partial_input_and_decodes_coalesced_packets() {
        let (mut transport, script) = transport(false);
        script
            .lock()
            .unwrap()
            .reads
            .extend([vec![3, 1], vec![2, 2, 9]]);
        assert!(transport.try_read().unwrap().is_none());
        assert_eq!(transport.try_read().unwrap().unwrap().as_ref(), &[3, 1, 2]);
        assert_eq!(transport.try_read().unwrap().unwrap().as_ref(), &[2, 9]);
    }
    #[test]
    fn udp_returns_complete_packets_before_reporting_a_bad_remainder() {
        for remainder in [vec![3, 2], vec![0]] {
            let (mut transport, script) = transport(true);
            let mut datagram = vec![2, 1, 2, 2];
            datagram.extend(remainder);
            script.lock().unwrap().reads.extend([datagram, vec![2, 9]]);
            assert_eq!(transport.try_read().unwrap().unwrap().as_ref(), &[2, 1]);
            assert_eq!(transport.try_read().unwrap().unwrap().as_ref(), &[2, 2]);
            // No subsequent datagram has been consumed to complete the remainder.
            assert_eq!(script.lock().unwrap().reads.len(), 1);
            assert!(transport.try_read().is_err());
            assert_eq!(transport.try_read().unwrap().unwrap().as_ref(), &[2, 9]);
        }
    }
    #[test]
    fn udp_drains_current_datagram_before_receiving_the_next() {
        let (mut transport, script) = transport(true);
        script
            .lock()
            .unwrap()
            .reads
            .extend([vec![2, 1, 2, 2], vec![2, 3]]);
        for value in [1, 2] {
            assert_eq!(transport.try_read().unwrap().unwrap().as_ref(), &[2, value]);
            assert_eq!(script.lock().unwrap().reads.len(), 1);
        }
        assert_eq!(transport.try_read().unwrap().unwrap().as_ref(), &[2, 3]);
    }
    #[test]
    fn empty_or_incomplete_udp_does_not_poison_the_next_datagram() {
        for bad in [vec![], vec![3, 1]] {
            let (mut transport, script) = transport(true);
            script.lock().unwrap().reads.extend([bad, vec![2, 9]]);
            assert!(transport.try_read().is_err());
            assert_eq!(transport.try_read().unwrap().unwrap().as_ref(), &[2, 9]);
        }
    }
    #[test]
    fn malformed_tcp_is_terminal() {
        let (mut transport, script) = transport(false);
        script.lock().unwrap().reads.extend([vec![0], vec![2, 1]]);
        assert!(transport.try_read().is_err());
        assert!(matches!(transport.try_read(), Err(Error::Disconnected)));
    }
}
