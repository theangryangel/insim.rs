use std::{
    collections::VecDeque,
    fmt::Debug,
    time::{Duration, Instant},
};

use insim_core::net::{self, ReceiveTransport};

use super::{
    Context, DatagramSocket, Decoder, InsimCodec, Interest, Runtime, Socket, StreamSocket,
    Transport,
    request::{self, Pending, Replies, Reply, RequestIds, Response},
    tap::{self, Filter, Policy, Sink, Tap, Taps},
};
#[cfg(any(feature = "tokio", feature = "blocking"))]
use crate::insim::Isi;
#[cfg(any(feature = "tokio", feature = "blocking"))]
use crate::insim::TinyType;
use crate::{Error, Packet, Result};

// Pause socket reads at capacity rather than dropping packets.
const INCOMING_LIMIT: usize = 16384;
const OUTGOING_LIMIT: usize = 1024 * 1024;

trait Control<R: Runtime>: ReceiveTransport<Runtime = R, Item = Packet> {
    fn queue(&mut self, packet: &Packet) -> net::Result<()>;
    fn flush(&mut self) -> net::Result<bool>;
    fn pending(&self) -> usize;
}
impl<S: Socket> Control<S::Runtime> for Transport<S, InsimCodec> {
    fn queue(&mut self, packet: &Packet) -> net::Result<()> {
        self.queue(packet)
    }
    fn flush(&mut self) -> net::Result<bool> {
        self.try_flush()
    }
    fn pending(&self) -> usize {
        self.pending_bytes()
    }
}
#[derive(Debug)]
struct Attached<T>(T);
impl<T: ReceiveTransport> ReceiveTransport for Attached<T>
where
    T::Item: Into<Packet>,
{
    type Runtime = T::Runtime;
    type Item = Packet;
    fn try_read(&mut self) -> net::Result<Option<Packet>> {
        self.0.try_read().map(|p| p.map(Into::into))
    }
    fn socket(&self) -> &dyn Socket<Runtime = T::Runtime> {
        self.0.socket()
    }
}

/// A link to LFS, rather than a race session.
///
/// The caller must drive `read` or `flush` for protocol and I/O progress. There is
/// no background task. Cancellation preserves accepted output and partial I/O;
/// it does not guarantee delivery if the session is dropped or stops being driven.
#[derive(Debug)]
pub struct Session<R: Runtime> {
    control: Box<dyn Control<R>>,
    attached: Vec<Box<dyn ReceiveTransport<Runtime = R, Item = Packet>>>,
    context: Context,
    // Keep recoverable errors in packet order.
    incoming: VecDeque<Result<Packet>>,
    // Deliver buffered packets before returning a terminal error.
    deferred: Option<Error>,
    next_source: usize,
    taps: Taps,
    request_ids: RequestIds,
}

/// An owned link whose InSim handshake has not yet started.
/// Receive-only listeners can be attached before initialization.
#[derive(Debug)]
pub struct Unstarted<R: Runtime> {
    session: Session<R>,
}
impl<R: Runtime> Session<R> {
    /// Take ownership of a TCP socket. Its type selects the runtime.
    pub fn tcp<S: StreamSocket<Runtime = R>>(socket: S) -> Result<Unstarted<R>> {
        Self::build(Transport::new(socket.into_tcp(), InsimCodec)?)
    }
    /// Take ownership of a connected UDP control socket. Its type selects the runtime.
    pub fn udp<S: DatagramSocket<Runtime = R>>(socket: S) -> Result<Unstarted<R>> {
        Self::build(Transport::new(socket.into_udp(), InsimCodec)?)
    }
    fn build(control: impl Control<R> + 'static) -> Result<Unstarted<R>> {
        Ok(Unstarted {
            session: Self {
                control: Box::new(control),
                attached: Vec::new(),
                context: Context::unstarted(Instant::now()),
                incoming: VecDeque::new(),
                deferred: None,
                next_source: 0,
                taps: Taps::default(),
                request_ids: RequestIds::default(),
            },
        })
    }
    /// Consume a standalone listener. Every attached socket must use this runtime.
    pub fn attach<T>(mut self, listener: T) -> Self
    where
        T: ReceiveTransport<Runtime = R> + 'static,
        T::Item: Into<Packet>,
    {
        self.attached.push(Box::new(Attached(listener)));
        self
    }
    /// Convenience construction for a UDP listener with a decoder or Demux.
    pub fn attach_udp<S, D>(self, socket: S, decoder: D) -> Result<Self>
    where
        S: DatagramSocket<Runtime = R>,
        D: Decoder,
        D::Item: Into<Packet>,
    {
        Ok(self.attach(net::Listener::udp(socket, decoder)?))
    }
    /// Accept a packet into persistent outgoing storage without waiting.
    /// Encoding failure leaves previously queued output intact.
    pub fn queue<P: Into<Packet>>(&mut self, packet: P) -> Result<()> {
        if !self.context.is_ready() {
            return Err(Error::Disconnected);
        }
        if self.control.pending() >= OUTGOING_LIMIT {
            return Err(Error::OutgoingOverflow);
        }
        self.queue_internal(&packet.into())
    }
    fn queue_internal(&mut self, packet: &Packet) -> Result<()> {
        if self.control.pending() >= OUTGOING_LIMIT + crate::MAX_SIZE_PACKET {
            self.context.close();
            return Err(Error::OutgoingOverflow);
        }
        self.control.queue(packet)?;
        Ok(())
    }
    /// Copy packets accepted by `filter` to a [`Tap`], according to `policy`.
    ///
    /// Matching packets are still returned by `read()`. The tap fills while the
    /// session is driven and ends when its policy is satisfied or the session ends.
    pub fn tap<F>(&mut self, filter: F, policy: Policy) -> Tap
    where
        F: FnMut(&Packet) -> bool + Send + 'static,
    {
        let (sink, tap) = tap::channel(policy);
        self.insert_tap(sink, Box::new(filter));
        tap
    }
    pub(super) fn insert_tap(&mut self, sink: Sink, filter: Filter) {
        self.taps.insert(sink, filter, Instant::now());
    }
    /// Set the ReqIs that `request` and `request_all` cycle through. The default is
    /// [`DEFAULT_REQUEST_IDS`](super::DEFAULT_REQUEST_IDS) (128–255). Use ReqIs
    /// outside this range for packets you send yourself. Requests already in
    /// flight keep their ReqIs.
    ///
    /// # Panics
    /// If `ids` is empty or includes 0, which LFS does not correlate.
    pub fn set_request_ids(&mut self, ids: std::ops::RangeInclusive<u8>) {
        self.request_ids = RequestIds::new(ids);
    }
    /// The ReqIs that `request` and `request_all` cycle through.
    pub fn request_ids(&self) -> std::ops::RangeInclusive<u8> {
        self.request_ids.range().clone()
    }
    /// Queue `packet` with the next ReqI from [`request_ids`](Self::request_ids) and
    /// return a ticket for the first reply of type `T` that echoes it.
    ///
    /// The ticket fills in while you keep driving the session, and the reply is
    /// still returned by `read()`.
    ///
    /// ```rust,no_run
    /// # async fn example(session: &mut insim::Session<insim::net::Tokio>) -> insim::Result<()> {
    /// use insim::insim::{Ism, TinyType};
    ///
    /// let mut ism = session.request::<Ism>(TinyType::Ism)?;
    /// loop {
    ///     tokio::select! {
    ///         packet = session.read() => println!("{:?}", packet?),
    ///         ism = &mut ism => {
    ///             println!("host: {:?}", ism?.hname);
    ///             break;
    ///         }
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn request<T: Response>(&mut self, packet: impl Into<Packet>) -> Result<Reply<T>> {
        let (pending, reply) = request::reply(packet.into());
        self.begin_request(pending)?;
        Ok(reply)
    }
    /// Like [`request`](Self::request), but collect every reply of type `T` until
    /// none has arrived for [`REQUEST_ALL_QUIET`](super::REQUEST_ALL_QUIET).
    pub fn request_all<T: Response>(&mut self, packet: impl Into<Packet>) -> Result<Replies<T>> {
        self.request_all_within(packet, super::REQUEST_ALL_QUIET)
    }
    /// Like [`request_all`](Self::request_all), with an explicit quiet period.
    pub fn request_all_within<T: Response>(
        &mut self,
        packet: impl Into<Packet>,
        quiet: Duration,
    ) -> Result<Replies<T>> {
        let (pending, replies) = request::replies(packet.into(), quiet);
        self.begin_request(pending)?;
        Ok(replies)
    }
    /// Stamp a request with the next reserved ReqI.
    pub(super) fn stamp(&mut self, pending: Pending) -> (Packet, Sink, Filter) {
        pending.start(self.request_ids.next())
    }
    /// Queue the request, then register its tap. Nothing is read in between.
    fn begin_request(&mut self, pending: Pending) -> Result<()> {
        let (packet, sink, filter) = self.stamp(pending);
        self.queue(packet)?;
        self.insert_tap(sink, filter);
        Ok(())
    }
    /// Earliest protocol or tap deadline.
    #[cfg(any(feature = "tokio", feature = "blocking"))]
    fn deadline(&self) -> Option<Instant> {
        match (self.context.deadline(), self.taps.deadline()) {
            (Some(a), Some(b)) => Some(a.min(b)),
            (a, b) => a.or(b),
        }
    }
    /// Attempt nonblocking progress and return one packet if available.
    /// This also advances protocol deadlines and pending writes. Malformed UDP
    /// input is logged and skipped; TCP and socket errors are returned.
    ///
    /// Packets already received are always returned before a terminal error.
    pub fn try_read(&mut self) -> Result<Option<Packet>> {
        if self.incoming.is_empty() {
            if let Some(error) = self.deferred.take() {
                return Err(error);
            }
            if let Err(error) = self.pump() {
                if self.incoming.is_empty() {
                    return Err(error);
                }
                self.deferred = Some(error);
            }
        } else if self.deferred.is_none()
            && let Err(error) = self.tick_and_flush()
        {
            self.deferred = Some(error);
        }
        self.incoming.pop_front().transpose()
    }
    fn is_full(&self) -> bool {
        self.incoming.len() >= INCOMING_LIMIT
    }
    fn tick_and_flush(&mut self) -> Result<()> {
        let result = self.tick_and_flush_inner();
        if result.is_err() {
            self.taps.close();
        }
        result
    }
    fn tick_and_flush_inner(&mut self) -> Result<()> {
        let now = Instant::now();
        self.taps.advance(now);
        if self.is_full() {
            // Backpressure must not trigger the inactivity timeout.
            self.context.hold(now);
        }
        if let Some(packet) = self.context.advance(now)? {
            self.queue_internal(&packet)?;
        }
        if let Err(error) = self.control.flush() {
            self.context.close();
            return Err(error.into());
        }
        Ok(())
    }
    fn pump(&mut self) -> Result<bool> {
        let result = self.pump_inner();
        if result.is_err() {
            self.taps.close();
        }
        result
    }
    fn pump_inner(&mut self) -> Result<bool> {
        self.tick_and_flush()?;
        if self.is_full() {
            return Ok(false);
        }
        let sources = 1 + self.attached.len();
        for offset in 0..sources {
            let index = (self.next_source + offset) % sources;
            let source: &mut dyn ReceiveTransport<Runtime = R, Item = Packet> = if index == 0 {
                &mut *self.control
            } else {
                &mut *self.attached[index - 1]
            };
            let datagram = source.socket().is_datagram();
            let packet = match source.try_read() {
                Ok(Some(packet)) => packet,
                Ok(None) => continue,
                // Preserve error ordering without failing handshake or flush.
                Err(error @ net::Error::IncompleteDecode { .. }) => {
                    self.next_source = (index + 1) % sources;
                    self.incoming.push_back(Err(error.into()));
                    return Ok(true);
                },
                Err(error)
                    if datagram
                        && matches!(error, net::Error::Frame(_) | net::Error::Decode { .. }) =>
                {
                    tracing::warn!(
                        source = index,
                        local_addr = ?source.socket().local_addr().ok(),
                        error = %error,
                        "Discarding malformed UDP input"
                    );
                    continue;
                },
                Err(error) => {
                    self.context.close();
                    return Err(error.into());
                },
            };
            self.next_source = (index + 1) % sources;
            let now = Instant::now();
            let response = self.context.receive(&packet, now);
            self.taps.offer(&packet, now);
            // Retain input if queuing or flushing the response fails.
            self.incoming.push_back(Ok(packet));
            if let Some(response) = response? {
                self.queue_internal(&response)?;
            }
            self.tick_and_flush()?;
            return Ok(true);
        }
        Ok(false)
    }
    #[cfg(any(feature = "tokio", feature = "blocking"))]
    fn interests(&self) -> Vec<(&dyn Socket<Runtime = R>, Interest)> {
        let read = !self.is_full();
        let control = Interest {
            read,
            write: self.control.pending() != 0,
        };
        let attached = Interest { read, write: false };
        std::iter::once((self.control.socket(), control))
            .chain(self.attached.iter().map(|t| (t.socket(), attached)))
            .collect()
    }
    #[cfg(any(feature = "tokio", feature = "blocking"))]
    fn begin_handshake(&mut self, isi: Isi) -> Result<()> {
        if isi.udpport != 0
            && !self.interests().iter().any(|(s, _)| {
                s.is_datagram() && s.local_addr().is_ok_and(|a| a.port() == isi.udpport)
            })
        {
            tracing::warn!(
                udpport = isi.udpport,
                "Isi UDP port has no matching local socket; this may be intentional with NAT"
            );
        }
        self.context = Context::handshake(
            isi.clone(),
            self.control.socket().is_datagram(),
            Instant::now(),
        )?;
        self.queue_internal(&isi.into())
    }
}
impl<R: Runtime> Unstarted<R> {
    /// Consume a listener to receive packets alongside the control transport.
    pub fn attach<T>(mut self, listener: T) -> Self
    where
        T: ReceiveTransport<Runtime = R> + 'static,
        T::Item: Into<Packet>,
    {
        self.session = self.session.attach(listener);
        self
    }
    /// Attach a socket directly using a decoder or Demux stack.
    pub fn attach_udp<S, D>(mut self, socket: S, decoder: D) -> Result<Self>
    where
        S: DatagramSocket<Runtime = R>,
        D: Decoder,
        D::Item: Into<Packet>,
    {
        self.session = self.session.attach_udp(socket, decoder)?;
        Ok(self)
    }
    /// Set the ReqIs that `request` and `request_all` will cycle through. See
    /// [`Session::set_request_ids`].
    ///
    /// # Panics
    /// If `ids` is empty or includes 0, which LFS does not correlate.
    pub fn with_request_ids(mut self, ids: std::ops::RangeInclusive<u8>) -> Self {
        self.session.set_request_ids(ids);
        self
    }
    /// Skip initialization for a conversation that is already running.
    pub fn resume(mut self) -> Session<R> {
        self.session.context = Context::resume(Instant::now());
        self.session
    }
    /// Receive without waiting before starting InSim.
    pub fn try_read(&mut self) -> Result<Option<Packet>> {
        self.session.try_read()
    }
}

#[cfg(any(feature = "tokio", feature = "blocking"))]
macro_rules! runtime_methods {
    ($runtime:ty, [$($async:tt)*], [$($await:tt)*]) => {
        impl Session<$runtime> {
            /// Wait for one packet. Dropping this future never discards I/O progress.
            pub $($async)* fn read(&mut self) -> Result<Packet> {
                loop {
                    if let Some(packet) = self.try_read()? { return Ok(packet); }
                    <$runtime>::wait(&self.interests(), self.deadline())$($await)*?;
                }
            }
            /// Drive the whole session until all queued output is accepted by the socket.
            pub $($async)* fn flush(&mut self) -> Result<()> {
                loop {
                    let progress = self.pump()?;
                    if self.control.pending() == 0 { return Ok(()); }
                    if !progress { <$runtime>::wait(&self.interests(), self.deadline())$($await)*?; }
                }
            }
            /// Queue once, then flush. Cancelling after acceptance leaves output queued.
            /// Retrying `write(packet)` can therefore duplicate that packet.
            pub $($async)* fn write<P: Into<Packet>>(&mut self, packet: P) -> Result<()> {
                self.queue(packet)?;
                self.flush()$($await)*
            }
            /// Send TINY_CLOSE, drain output and close the TCP write direction.
            pub $($async)* fn shutdown(&mut self) -> Result<()> {
                self.write(TinyType::Close)$($await)*?;
                self.control.socket().shutdown()?;
                self.context.close();
                self.taps.close();
                Ok(())
            }
        }
        impl Unstarted<$runtime> {
            /// Initialize InSim and retain the matching Ver for the next `read`.
            /// ReqI must be nonzero. Dropping this owning operation drops the link.
            pub $($async)* fn handshake(mut self, isi: Isi) -> Result<Session<$runtime>> {
                self.session.begin_handshake(isi)?;
                loop {
                    let progress = self.session.pump()?;
                    if self.session.context.is_ready() && self.session.control.pending() == 0 {
                        return Ok(self.session);
                    }
                    if !progress {
                        <$runtime>::wait(&self.session.interests(), self.session.deadline())$($await)*?;
                    }
                }
            }
            /// Receive telemetry before initialization.
            pub $($async)* fn read(&mut self) -> Result<Packet> { self.session.read()$($await)* }
        }
    };
}
/// Progress without delivery, for a driver that forwards packets elsewhere.
#[cfg(feature = "tokio")]
impl Session<super::Tokio> {
    /// Read, reply and flush until stalled. A terminal error is kept until the
    /// packets before it have been taken.
    pub(super) fn fill(&mut self) {
        while self.deferred.is_none() {
            match self.pump() {
                Ok(true) => continue,
                Ok(false) => return,
                Err(error) => self.deferred = Some(error),
            }
        }
    }
    /// The next packet or error, in order, and whether it ends the session.
    pub(super) fn take(&mut self) -> Option<(Result<Packet>, bool)> {
        match self.incoming.pop_front() {
            Some(item) => Some((item, false)),
            None => self.deferred.take().map(|error| (Err(error), true)),
        }
    }
    pub(super) fn has_output(&self) -> bool {
        !self.incoming.is_empty() || self.deferred.is_some()
    }
    pub(super) fn can_queue(&self) -> bool {
        self.control.pending() < OUTGOING_LIMIT
    }
    /// Wait for readiness or a deadline. A wait failure ends the session.
    pub(super) async fn wait(&mut self) {
        if let Err(error) = super::Tokio::wait(&self.interests(), self.deadline()).await {
            self.context.close();
            self.taps.close();
            let _ = self.deferred.get_or_insert(error.into());
        }
    }
}

#[cfg(feature = "blocking")]
runtime_methods!(super::Blocking, [], []);
#[cfg(feature = "tokio")]
runtime_methods!(super::Tokio, [async], [.await]);

#[cfg(all(test, feature = "tokio"))]
mod tests {
    use std::{
        future::Future,
        io,
        net::SocketAddr,
        sync::{Arc, Mutex},
        task::{Context as TaskContext, Poll, Waker},
    };

    use bytes::BytesMut;
    use insim_core::net::Encoder;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    use super::*;
    use crate::{
        VERSION,
        identifiers::RequestId,
        insim::{Tiny, Ver},
    };

    #[derive(Debug, Default)]
    struct Script {
        blocked: bool,
        first: bool,
        sent: Vec<u8>,
        input: Option<Vec<u8>>,
        following: VecDeque<Vec<u8>>,
        input_error: Option<io::ErrorKind>,
    }
    #[derive(Debug)]
    struct Fake {
        datagram: bool,
        script: Arc<Mutex<Script>>,
    }
    impl Socket for Fake {
        type Runtime = super::super::Tokio;
        fn is_datagram(&self) -> bool {
            self.datagram
        }
        fn prepare(&self) -> io::Result<()> {
            Ok(())
        }
        fn try_recv(&self, buf: &mut [u8]) -> io::Result<usize> {
            let mut script = self.script.lock().unwrap();
            if let Some(error) = script.input_error.take() {
                return Err(error.into());
            }
            let Some(input) = script.input.take().or_else(|| script.following.pop_front()) else {
                return Err(io::ErrorKind::WouldBlock.into());
            };
            buf[..input.len()].copy_from_slice(&input);
            Ok(input.len())
        }
        fn try_send(&self, bytes: &[u8]) -> io::Result<usize> {
            let mut script = self.script.lock().unwrap();
            if script.blocked {
                if !self.datagram && script.first {
                    script.first = false;
                    script.sent.extend_from_slice(&bytes[..2]);
                    return Ok(2);
                }
                return Err(io::ErrorKind::WouldBlock.into());
            }
            script.sent.extend_from_slice(bytes);
            Ok(bytes.len())
        }
        fn local_addr(&self) -> io::Result<SocketAddr> {
            Ok("127.0.0.1:1".parse().unwrap())
        }
        fn shutdown(&self) -> io::Result<()> {
            Ok(())
        }
        fn poll_ready(&self, _: &mut TaskContext<'_>, interest: Interest) -> Poll<io::Result<()>> {
            let script = self.script.lock().unwrap();
            if (interest.read
                && (script.input.is_some()
                    || !script.following.is_empty()
                    || script.input_error.is_some()))
                || (interest.write && !script.blocked)
            {
                Poll::Ready(Ok(()))
            } else {
                Poll::Pending
            }
        }
    }
    #[tokio::test]
    async fn cancelling_write_preserves_tcp_suffix_and_whole_udp_datagram() {
        for datagram in [false, true] {
            let script = Arc::new(Mutex::new(Script {
                blocked: true,
                first: true,
                ..Script::default()
            }));
            let fake = Fake {
                datagram,
                script: script.clone(),
            };
            let mut session = Session::build(Transport::new(fake, InsimCodec).unwrap())
                .unwrap()
                .resume();
            {
                let mut write = std::pin::pin!(session.write(Tiny {
                    reqi: RequestId(2),
                    subt: TinyType::Ping
                }));
                assert!(
                    write
                        .as_mut()
                        .poll(&mut TaskContext::from_waker(Waker::noop()))
                        .is_pending()
                );
            }
            assert_eq!(session.control.pending(), if datagram { 4 } else { 2 });
            script.lock().unwrap().blocked = false;
            session.flush().await.unwrap();
            assert_eq!(script.lock().unwrap().sent, [1, 3, 2, 3]);
        }
    }
    #[tokio::test]
    async fn automatic_udp_reply_survives_cancelled_read() {
        let script = Arc::new(Mutex::new(Script {
            blocked: true,
            input: Some(vec![1, 3, 0, 0]),
            ..Script::default()
        }));
        let mut session = Session::build(
            Transport::new(
                Fake {
                    datagram: true,
                    script: script.clone(),
                },
                InsimCodec,
            )
            .unwrap(),
        )
        .unwrap()
        .resume();
        assert!(matches!(session.read().await.unwrap(), Packet::Tiny(_)));
        assert_eq!(session.control.pending(), 4);
        {
            let mut read = std::pin::pin!(session.read());
            assert!(
                read.as_mut()
                    .poll(&mut TaskContext::from_waker(Waker::noop()))
                    .is_pending()
            );
        }
        script.lock().unwrap().blocked = false;
        session.flush().await.unwrap();
        assert_eq!(script.lock().unwrap().sent, [1, 3, 0, 0]);
    }
    #[test]
    fn udp_packet_reads_preserve_outgauge_and_both_outsim_formats() {
        use outsim::outsimpack2::OutSimOpts;

        use super::super::{Demux, OutGaugeCodec, OutSimCodec};

        let script = Arc::new(Mutex::new(Script::default()));
        let fake = || Fake {
            datagram: true,
            script: script.clone(),
        };
        let mut transport = Transport::new(
            fake(),
            Demux::new()
                .with(OutGaugeCodec)
                .with(OutSimCodec::default())
                .with(InsimCodec),
        )
        .unwrap();
        let mut gauge = vec![0; 92];
        gauge[4..8].copy_from_slice(b"XFG\0");
        script.lock().unwrap().input = Some(gauge);
        assert!(matches!(
            transport.try_read().unwrap(),
            Some(Packet::Outgauge(_))
        ));
        for len in [64, 68] {
            script.lock().unwrap().input = Some(vec![0; len]);
            assert!(matches!(
                transport.try_read().unwrap(),
                Some(Packet::Outsim(_))
            ));
        }
        script.lock().unwrap().input = Some(vec![1, 3, 2, 3, 1, 3, 4, 3]);
        for reqi in [2, 4] {
            assert!(matches!(transport.try_read().unwrap(),
                Some(Packet::Tiny(Tiny { reqi: RequestId(id), .. })) if id == reqi));
        }
        assert!(transport.try_read().unwrap().is_none());

        let mut extended = Transport::new(
            fake(),
            Demux::new()
                .with(OutGaugeCodec)
                .with(OutSimCodec::extended(OutSimOpts::all()))
                .with(InsimCodec),
        )
        .unwrap();
        let mut bytes = vec![0; 280];
        bytes[..4].copy_from_slice(b"OSIM");
        script.lock().unwrap().input = Some(bytes);
        assert!(matches!(
            extended.try_read().unwrap(),
            Some(Packet::Outsim2(_))
        ));
        assert!(extended.try_read().unwrap().is_none());
    }
    #[test]
    fn udp_keepalive_is_processed_before_a_malformed_remainder() {
        let script = Arc::new(Mutex::new(Script {
            input: Some(vec![1, 3, 0, 0, 1, 3]),
            ..Script::default()
        }));
        let mut session = Session::build(
            Transport::new(
                Fake {
                    datagram: true,
                    script: script.clone(),
                },
                InsimCodec,
            )
            .unwrap(),
        )
        .unwrap()
        .resume();
        assert!(matches!(session.try_read().unwrap(), Some(Packet::Tiny(_))));
        assert_eq!(script.lock().unwrap().sent, [1, 3, 0, 0]);
        assert!(session.try_read().unwrap().is_none());
        script.lock().unwrap().input = Some(vec![1, 3, 2, 3]);
        assert!(matches!(
            session.try_read().unwrap(),
            Some(Packet::Tiny(Tiny {
                reqi: RequestId(2),
                ..
            }))
        ));
    }
    fn bad_udp_inputs() -> [Vec<u8>; 2] {
        [vec![0], vec![1, 3, 0, 255]] // Invalid framing; valid framing with invalid Tiny subtype.
    }
    #[tokio::test]
    async fn malformed_udp_does_not_abort_handshake() {
        for bad in bad_udp_inputs() {
            let mut version = BytesMut::new();
            InsimCodec
                .encode(
                    &Ver {
                        reqi: RequestId(7),
                        insimver: VERSION,
                        ..Ver::default()
                    }
                    .into(),
                    &mut version,
                )
                .unwrap();
            let script = Arc::new(Mutex::new(Script {
                input: Some(bad),
                following: VecDeque::from([version.to_vec()]),
                ..Script::default()
            }));
            let unstarted = Session::build(
                Transport::new(
                    Fake {
                        datagram: true,
                        script: script.clone(),
                    },
                    InsimCodec,
                )
                .unwrap(),
            )
            .unwrap();
            let isi = Isi {
                reqi: RequestId(7),
                ..Isi::default()
            };
            let mut expected = BytesMut::new();
            InsimCodec
                .encode(&isi.clone().into(), &mut expected)
                .unwrap();
            let mut session = unstarted.handshake(isi).await.unwrap();
            assert!(matches!(
                session.read().await.unwrap(),
                Packet::Ver(Ver {
                    reqi: RequestId(7),
                    ..
                })
            ));
            assert_eq!(script.lock().unwrap().sent, expected.as_ref());
        }
    }
    #[tokio::test]
    async fn malformed_attached_udp_does_not_fail_or_repeat_an_accepted_write() {
        for datagram in [false, true] {
            for bad in bad_udp_inputs() {
                let control = Arc::new(Mutex::new(Script::default()));
                let udp = Arc::new(Mutex::new(Script {
                    input: Some(bad),
                    ..Script::default()
                }));
                let mut session = Session::build(
                    Transport::new(
                        Fake {
                            datagram,
                            script: control.clone(),
                        },
                        InsimCodec,
                    )
                    .unwrap(),
                )
                .unwrap()
                .resume()
                .attach(
                    Transport::new(
                        Fake {
                            datagram: true,
                            script: udp.clone(),
                        },
                        InsimCodec,
                    )
                    .unwrap(),
                );
                session
                    .write(Tiny {
                        reqi: RequestId(2),
                        subt: TinyType::Ping,
                    })
                    .await
                    .unwrap();
                session.flush().await.unwrap();
                assert_eq!(control.lock().unwrap().sent, [1, 3, 2, 3]);
                udp.lock().unwrap().input = Some(vec![1, 3, 3, 3]);
                assert!(matches!(
                    session.read().await.unwrap(),
                    Packet::Tiny(Tiny {
                        reqi: RequestId(3),
                        ..
                    })
                ));
            }
        }
    }
    #[test]
    fn malformed_udp_does_not_hide_ready_input_on_another_socket() {
        let control = Arc::new(Mutex::new(Script {
            input: Some(vec![0]),
            ..Script::default()
        }));
        let udp = Arc::new(Mutex::new(Script {
            input: Some(vec![1, 3, 4, 3]),
            ..Script::default()
        }));
        let mut session = Session::build(
            Transport::new(
                Fake {
                    datagram: true,
                    script: control,
                },
                InsimCodec,
            )
            .unwrap(),
        )
        .unwrap()
        .resume()
        .attach(
            Transport::new(
                Fake {
                    datagram: true,
                    script: udp,
                },
                InsimCodec,
            )
            .unwrap(),
        );
        assert!(matches!(
            session.try_read().unwrap(),
            Some(Packet::Tiny(Tiny {
                reqi: RequestId(4),
                ..
            }))
        ));
    }
    #[test]
    fn tcp_decode_and_udp_socket_errors_remain_terminal() {
        let script = Arc::new(Mutex::new(Script {
            input: Some(vec![0]),
            ..Script::default()
        }));
        let mut tcp = Session::build(
            Transport::new(
                Fake {
                    datagram: false,
                    script,
                },
                InsimCodec,
            )
            .unwrap(),
        )
        .unwrap()
        .resume();
        assert!(matches!(
            tcp.try_read(),
            Err(Error::Transport(net::Error::Frame(_)))
        ));
        assert!(matches!(tcp.try_read(), Err(Error::Disconnected)));

        let script = Arc::new(Mutex::new(Script {
            input_error: Some(io::ErrorKind::ConnectionReset),
            ..Script::default()
        }));
        let mut udp = Session::build(
            Transport::new(
                Fake {
                    datagram: true,
                    script,
                },
                InsimCodec,
            )
            .unwrap(),
        )
        .unwrap()
        .resume();
        assert!(
            matches!(udp.try_read(), Err(Error::IO(error)) if error.kind() == io::ErrorKind::ConnectionReset)
        );
        assert!(matches!(udp.try_read(), Err(Error::Disconnected)));
    }
    #[test]
    fn incomplete_decode_is_reported_in_order_and_is_not_terminal() {
        for datagram in [false, true] {
            // Tiny(reqi 2), a Tiny frame padded to 8 bytes, then Tiny(reqi 4).
            let script = Arc::new(Mutex::new(Script {
                input: Some(vec![1, 3, 2, 3, 2, 3, 9, 3, 0, 0, 0, 0, 1, 3, 4, 3]),
                ..Script::default()
            }));
            let mut session = Session::build(
                Transport::new(
                    Fake {
                        datagram,
                        script: script.clone(),
                    },
                    InsimCodec,
                )
                .unwrap(),
            )
            .unwrap()
            .resume();
            assert!(matches!(
                session.try_read().unwrap(),
                Some(Packet::Tiny(Tiny {
                    reqi: RequestId(2),
                    ..
                }))
            ));
            assert!(matches!(
                session.try_read(),
                Err(Error::IncompleteDecode { remaining, .. }) if remaining.len() == 4
            ));
            assert!(matches!(
                session.try_read().unwrap(),
                Some(Packet::Tiny(Tiny {
                    reqi: RequestId(4),
                    ..
                }))
            ));
        }
    }
    #[tokio::test]
    async fn full_buffer_pauses_reading_without_blocking_writes() {
        let script = Arc::new(Mutex::new(Script {
            input: Some(vec![1, 3, 9, 3]),
            ..Script::default()
        }));
        let mut session = Session::build(
            Transport::new(
                Fake {
                    datagram: false,
                    script: script.clone(),
                },
                InsimCodec,
            )
            .unwrap(),
        )
        .unwrap()
        .resume();
        for _ in 0..INCOMING_LIMIT {
            session.incoming.push_back(Ok(TinyType::Ping.into()));
        }
        session.write(TinyType::Ping).await.unwrap();
        assert_eq!(script.lock().unwrap().sent, [1, 3, 0, 3]);
        // The ready input was left in the socket rather than overflowing the buffer.
        assert!(script.lock().unwrap().input.is_some());
        assert_eq!(session.incoming.len(), INCOMING_LIMIT);
        assert!(session.interests().iter().all(|(_, i)| !i.read));
        // Draining resumes reading, and the held packet arrives in order.
        session.incoming.truncate(1);
        assert!(session.try_read().unwrap().is_some());
        assert!(matches!(
            session.try_read().unwrap(),
            Some(Packet::Tiny(Tiny {
                reqi: RequestId(9),
                ..
            }))
        ));
    }
    #[test]
    fn buffered_packets_are_delivered_before_a_terminal_error() {
        let script = Arc::new(Mutex::new(Script::default()));
        let mut session = Session::build(
            Transport::new(
                Fake {
                    datagram: false,
                    script,
                },
                InsimCodec,
            )
            .unwrap(),
        )
        .unwrap()
        .resume();
        for reqi in [2, 4] {
            session.incoming.push_back(Ok(Tiny {
                reqi: RequestId(reqi),
                subt: TinyType::Ping,
            }
            .into()));
        }
        // A closed context makes the tick fail while packets are still buffered.
        session.context.close();
        for reqi in [2, 4] {
            assert!(matches!(
                session.try_read().unwrap(),
                Some(Packet::Tiny(Tiny { reqi: RequestId(id), .. })) if id == reqi
            ));
        }
        assert!(matches!(session.try_read(), Err(Error::Disconnected)));
    }
    #[tokio::test]
    async fn tcp_handshake_retains_ver_and_attaches_a_mixed_udp_listener() {
        tokio::time::timeout(std::time::Duration::from_secs(3), async {
            let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
            let stream = tokio::net::TcpStream::connect(listener.local_addr().unwrap())
                .await
                .unwrap();
            let (mut lfs, _) = listener.accept().await.unwrap();
            let udp = tokio::net::UdpSocket::bind("127.0.0.1:0").await.unwrap();
            let udp_addr = udp.local_addr().unwrap();
            let source = net::Listener::udp(
                udp,
                super::super::Demux::new()
                    .with(outgauge::OutGaugeCodec)
                    .with(InsimCodec),
            )
            .unwrap();
            let peer = async {
                let mut isi = [0; 44];
                let _ = lfs.read_exact(&mut isi).await.unwrap();
                assert_eq!(isi[2], 7);
                let mut bytes = BytesMut::new();
                InsimCodec
                    .encode(
                        &Ver {
                            reqi: RequestId(7),
                            insimver: VERSION,
                            ..Ver::default()
                        }
                        .into(),
                        &mut bytes,
                    )
                    .unwrap();
                lfs.write_all(&bytes[..3]).await.unwrap();
                tokio::task::yield_now().await;
                lfs.write_all(&bytes[3..]).await.unwrap();
                // Keep TCP open while the session reads attached datagrams.
                let mut close = [0; 4];
                let _ = lfs.read_exact(&mut close).await.unwrap();
                assert_eq!(close, [1, 3, 0, 2]);
            };
            let client = async {
                let mut session = Session::tcp(stream)
                    .unwrap()
                    .attach(source)
                    .handshake(Isi {
                        reqi: RequestId(7),
                        udpport: udp_addr.port(),
                        ..Isi::default()
                    })
                    .await
                    .unwrap();
                assert!(matches!(session.read().await.unwrap(), Packet::Ver(_)));
                let telemetry = tokio::net::UdpSocket::bind("127.0.0.1:0").await.unwrap();
                let _ = telemetry.send_to(&[1, 3, 2, 3], udp_addr).await.unwrap();
                assert!(matches!(
                    session.read().await.unwrap(),
                    Packet::Tiny(Tiny {
                        reqi: RequestId(2),
                        ..
                    })
                ));
                // A real telemetry packet follows an InSim packet on the same socket.
                let mut gauge = vec![0; 92];
                gauge[4..8].copy_from_slice(b"XFG\0");
                let _ = telemetry.send_to(&gauge, udp_addr).await.unwrap();
                assert!(matches!(session.read().await.unwrap(), Packet::Outgauge(_)));
                session.shutdown().await.unwrap();
            };
            tokio::join!(peer, client);
        })
        .await
        .unwrap();
    }
}

#[cfg(all(test, feature = "blocking"))]
mod blocking_tests {
    use std::{
        io::{Read, Write},
        net::{TcpListener, TcpStream, UdpSocket},
        time::Duration,
    };

    use insim_core::net::Encoder;

    use super::*;
    use crate::{
        VERSION,
        identifiers::RequestId,
        insim::{Tiny, Ver},
    };

    #[test]
    fn blocking_handshake_waits_on_tcp_and_an_attached_listener() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let stream = TcpStream::connect(listener.local_addr().unwrap()).unwrap();
        let (mut lfs, _) = listener.accept().unwrap();
        lfs.set_read_timeout(Some(Duration::from_secs(3))).unwrap();
        let udp = UdpSocket::bind("127.0.0.1:0").unwrap();
        let address = udp.local_addr().unwrap();
        let peer = std::thread::spawn(move || {
            let mut isi = [0; 44];
            lfs.read_exact(&mut isi).unwrap();
            let mut bytes = bytes::BytesMut::new();
            InsimCodec
                .encode(
                    &Ver {
                        reqi: RequestId(9),
                        insimver: VERSION,
                        ..Ver::default()
                    }
                    .into(),
                    &mut bytes,
                )
                .unwrap();
            lfs.write_all(&bytes).unwrap();
            let sender = UdpSocket::bind("127.0.0.1:0").unwrap();
            assert_eq!(sender.send_to(&[1, 3, 0, 0], address).unwrap(), 4);
            let mut reply = [0; 4];
            lfs.read_exact(&mut reply).unwrap();
            assert_eq!(reply, [1, 3, 0, 0]);
            lfs.read_exact(&mut reply).unwrap();
            assert_eq!(reply, [1, 3, 0, 2]);
        });
        let mut session = Session::tcp(stream)
            .unwrap()
            .attach(net::Listener::udp(udp, InsimCodec).unwrap())
            .handshake(Isi {
                reqi: RequestId(9),
                udpport: address.port(),
                ..Isi::default()
            })
            .unwrap();
        let mut saw_ver = false;
        let mut saw_keepalive = false;
        for _ in 0..2 {
            match session.read().unwrap() {
                Packet::Ver(_) => saw_ver = true,
                Packet::Tiny(Tiny {
                    subt: TinyType::None,
                    ..
                }) => saw_keepalive = true,
                packet => panic!("unexpected {packet:?}"),
            }
        }
        assert!(saw_ver && saw_keepalive);
        session.shutdown().unwrap();
        peer.join().unwrap();
    }
}

#[cfg(all(test, feature = "tokio"))]
mod udp_tests {
    use std::time::Duration;

    use insim_core::net::Encoder;
    use tokio::net::UdpSocket;

    use super::*;
    use crate::{
        VERSION,
        identifiers::RequestId,
        insim::{Tiny, Ver},
    };

    #[tokio::test]
    async fn udp_control_uses_attached_replies_and_recovers_from_bad_datagrams() {
        tokio::time::timeout(Duration::from_secs(3), async {
            let lfs = UdpSocket::bind("127.0.0.1:0").await.unwrap();
            let control = UdpSocket::bind("127.0.0.1:0").await.unwrap();
            control.connect(lfs.local_addr().unwrap()).await.unwrap();
            let replies = UdpSocket::bind("127.0.0.1:0").await.unwrap();
            let reply_addr = replies.local_addr().unwrap();
            let peer = async {
                let mut input = [0; 1024];
                let (n, control_addr) = lfs.recv_from(&mut input).await.unwrap();
                assert_eq!(n, 44);
                assert_eq!(input[2], 5);
                let mut bytes = bytes::BytesMut::new();
                InsimCodec
                    .encode(
                        &Ver {
                            reqi: RequestId(5),
                            insimver: VERSION,
                            ..Ver::default()
                        }
                        .into(),
                        &mut bytes,
                    )
                    .unwrap();
                assert_eq!(lfs.send_to(&bytes, reply_addr).await.unwrap(), bytes.len());
                assert_eq!(lfs.send_to(&[1, 3, 0, 0], reply_addr).await.unwrap(), 4);
                let (n, from) = lfs.recv_from(&mut input).await.unwrap();
                assert_eq!(from, control_addr);
                assert_eq!(&input[..n], &[1, 3, 0, 0]);
                // A malformed control datagram must not poison the following one.
                assert_eq!(lfs.send_to(&[1, 3], control_addr).await.unwrap(), 2);
                assert_eq!(lfs.send_to(&[1, 3, 9, 3], control_addr).await.unwrap(), 4);
            };
            let client = async {
                let mut session = Session::udp(control)
                    .unwrap()
                    .attach_udp(replies, InsimCodec)
                    .unwrap()
                    .handshake(Isi {
                        reqi: RequestId(5),
                        udpport: reply_addr.port(),
                        ..Isi::default()
                    })
                    .await
                    .unwrap();
                assert!(matches!(session.read().await.unwrap(), Packet::Ver(_)));
                assert!(matches!(
                    session.read().await.unwrap(),
                    Packet::Tiny(Tiny {
                        subt: TinyType::None,
                        ..
                    })
                ));
                assert!(matches!(
                    session.read().await.unwrap(),
                    Packet::Tiny(Tiny {
                        reqi: RequestId(9),
                        subt: TinyType::Ping
                    })
                ));
            };
            tokio::join!(peer, client);
        })
        .await
        .unwrap();
    }
}

#[cfg(all(test, feature = "blocking"))]
mod blocking_request_tests {
    use std::{
        io::{Read, Write},
        net::{TcpListener, TcpStream},
        time::Duration,
    };

    use bytes::BytesMut;
    use insim_core::net::{Decoder, Encoder};

    use super::*;
    use crate::{
        identifiers::RequestId,
        insim::{Ism, TinyType},
    };

    #[test]
    fn blocking_request_is_checked_between_reads() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let stream = TcpStream::connect(listener.local_addr().unwrap()).unwrap();
        let (mut lfs, _) = listener.accept().unwrap();
        lfs.set_read_timeout(Some(Duration::from_secs(3))).unwrap();
        let peer = std::thread::spawn(move || {
            let mut input = BytesMut::new();
            let request = loop {
                if let Some(packet) = InsimCodec.decode(&mut input).unwrap() {
                    break packet;
                }
                let mut buf = [0; 64];
                let n = lfs.read(&mut buf).unwrap();
                input.extend_from_slice(&buf[..n]);
            };
            let mut bytes = BytesMut::new();
            InsimCodec
                .encode(
                    &Ism {
                        reqi: request.request_id().unwrap(),
                        ..Default::default()
                    }
                    .into(),
                    &mut bytes,
                )
                .unwrap();
            lfs.write_all(&bytes).unwrap();
            lfs
        });
        let mut session = Session::tcp(stream).unwrap().resume();
        let mut ism = session.request::<Ism>(TinyType::Ism).unwrap();
        assert!(ism.try_get().is_none());
        let packet = session.read().unwrap();
        let reply = ism.try_get().unwrap().unwrap();
        assert!(matches!(packet, Packet::Ism(_)));
        assert!(session.request_ids().contains(&reply.reqi.0));
        assert_ne!(reply.reqi, RequestId(0));
        drop(peer.join().unwrap());
    }
}
