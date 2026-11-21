//! A split Tokio session: a driver task owns the session and keeps reading, while
//! cloneable [`Sender`]s and one [`Receiver`] talk to it over channels. This is
//! the same idea as tower's `Buffer` and hyper's `SendRequest`.

use std::{
    fmt,
    future::Future,
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};

use tokio::sync::{mpsc, oneshot};

use super::{
    Session, Tokio,
    request::{self, Pending, Response},
    tap::{self, Filter, Policy, Sink, Tap},
};
use crate::{Error, Packet, Result};

enum Command {
    Packet(Packet, Option<oneshot::Sender<Result<()>>>),
    Tap(Sink, Filter),
    Request(Pending),
    Shutdown(oneshot::Sender<Result<()>>),
}

impl Session<Tokio> {
    /// Split into a cloneable [`Sender`], a [`Receiver`], and a [`Driver`] that owns
    /// the session. Spawn the driver.
    ///
    /// The driver reads continuously, so keepalives are answered and taps fill even
    /// while the receiver is not being polled. `capacity` (at least 1) bounds both
    /// the command queue and the packets waiting for the receiver. When the receiver
    /// falls behind, the driver keeps reading into the session's own buffer and
    /// pauses only once that buffer is full, as `read()` does. If you only use
    /// taps and requests, drop the receiver: the driver then discards packets
    /// once taps have copied them, instead of eventually pausing.
    ///
    /// ```rust,no_run
    /// # async fn example(session: insim::Session<insim::net::Tokio>) -> insim::Result<()> {
    /// use insim::insim::{Ism, Ncn, TinyType};
    ///
    /// let (sender, mut receiver, driver) = session.into_split(256);
    /// let _ = tokio::spawn(driver);
    ///
    /// sender.send(TinyType::Ping).await?; // cloneable; hand it to anything
    /// let ism: Ism = sender.request(TinyType::Ism).await?;
    /// let ncns: Vec<Ncn> = sender.request_all(TinyType::Ncn).await?;
    ///
    /// while let Some(packet) = receiver.recv().await {
    ///     println!("{:?}", packet?);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn into_split(self, capacity: usize) -> (Sender, Receiver, Driver) {
        let (commands_tx, commands) = mpsc::channel(capacity);
        let (events_tx, events) = mpsc::channel(capacity);
        (
            Sender {
                commands: commands_tx,
            },
            Receiver { events },
            Driver {
                inner: Box::pin(drive(self, commands, events_tx)),
            },
        )
    }
}

/// Sends packets and requests to a split session. Cheap to clone.
#[derive(Debug, Clone)]
pub struct Sender {
    commands: mpsc::Sender<Command>,
}
impl Sender {
    async fn command(&self, command: Command) -> Result<()> {
        self.commands
            .send(command)
            .await
            .map_err(|_| Error::Disconnected)
    }
    /// Wait until the session has accepted `packet` into its outgoing buffer, as
    /// `Session::queue` does. Encoding errors are returned here.
    pub async fn send<P: Into<Packet>>(&self, packet: P) -> Result<()> {
        let (ack, accepted) = oneshot::channel();
        self.command(Command::Packet(packet.into(), Some(ack)))
            .await?;
        accepted.await.unwrap_or(Err(Error::Disconnected))
    }
    /// Queue `packet` without waiting. Fails with [`Error::OutgoingOverflow`] when
    /// the command queue is full. The driver logs and drops packets it cannot encode.
    pub fn try_send<P: Into<Packet>>(&self, packet: P) -> Result<()> {
        self.commands
            .try_send(Command::Packet(packet.into(), None))
            .map_err(|error| match error {
                mpsc::error::TrySendError::Full(_) => Error::OutgoingOverflow,
                mpsc::error::TrySendError::Closed(_) => Error::Disconnected,
            })
    }
    /// See `Session::tap`. Packets that arrive before the driver registers the
    /// tap are not copied.
    pub async fn tap<F>(&self, filter: F, policy: Policy) -> Result<Tap>
    where
        F: FnMut(&Packet) -> bool + Send + 'static,
    {
        let (sink, tap) = tap::channel(policy);
        self.command(Command::Tap(sink, Box::new(filter))).await?;
        Ok(tap)
    }
    /// Send `packet` with a reserved ReqI and wait for the first reply of type `T`.
    /// See `Session::request`. LFS does not answer every request, so consider a timeout.
    pub async fn request<T: Response>(&self, packet: impl Into<Packet>) -> Result<T> {
        let (pending, reply) = request::reply(packet.into());
        self.command(Command::Request(pending)).await?;
        reply.await
    }
    /// Send `packet` with a reserved ReqI and collect every reply of type `T` until
    /// none has arrived for [`REQUEST_ALL_QUIET`](super::REQUEST_ALL_QUIET).
    pub async fn request_all<T: Response>(&self, packet: impl Into<Packet>) -> Result<Vec<T>> {
        self.request_all_within(packet, super::REQUEST_ALL_QUIET)
            .await
    }
    /// Like [`request_all`](Self::request_all), with an explicit quiet period.
    pub async fn request_all_within<T: Response>(
        &self,
        packet: impl Into<Packet>,
        quiet: Duration,
    ) -> Result<Vec<T>> {
        let (pending, replies) = request::replies(packet.into(), quiet);
        self.command(Command::Request(pending)).await?;
        replies.await
    }
    /// Send TINY_CLOSE after everything already accepted, and close the TCP write
    /// direction. The receiver gets the packets read so far, then `None`.
    pub async fn shutdown(&self) -> Result<()> {
        let (ack, done) = oneshot::channel();
        self.command(Command::Shutdown(ack)).await?;
        done.await.unwrap_or(Err(Error::Disconnected))
    }
}

/// Receives everything a split session reads, in order.
#[derive(Debug)]
pub struct Receiver {
    events: mpsc::Receiver<Result<Packet>>,
}
impl Receiver {
    /// The next packet or error. Recoverable errors such as
    /// [`Error::IncompleteDecode`] arrive in order. A terminal error is last, and
    /// `None` follows once the session has ended.
    pub async fn recv(&mut self) -> Option<Result<Packet>> {
        self.events.recv().await
    }
}

/// Owns and drives a split session. Spawn it, or await it alongside other work.
///
/// It finishes when the session ends, after a shutdown, or when every sender
/// and the receiver have been dropped (after flushing accepted output). It returns
/// an error only if nobody else saw it: a terminal error goes to the receiver if
/// it still exists.
#[must_use = "the driver does nothing unless spawned or awaited"]
pub struct Driver {
    inner: Pin<Box<dyn Future<Output = Result<()>> + Send>>,
}
impl Future for Driver {
    type Output = Result<()>;
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
        self.inner.as_mut().poll(cx)
    }
}
impl fmt::Debug for Driver {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Driver").finish_non_exhaustive()
    }
}

enum Wake<'a> {
    Ready,
    Command(Option<Command>),
    Permit(Option<mpsc::Permit<'a, Result<Packet>>>),
}

async fn drive(
    mut session: Session<Tokio>,
    mut commands: mpsc::Receiver<Command>,
    events: mpsc::Sender<Result<Packet>>,
) -> Result<()> {
    let mut senders = true;
    loop {
        // Commands already waiting, while the session has room for their output.
        while senders && session.can_queue() {
            match commands.try_recv() {
                Ok(command) => {
                    if apply(&mut session, command).await {
                        return finish(session, &events).await;
                    }
                },
                Err(mpsc::error::TryRecvError::Empty) => break,
                Err(mpsc::error::TryRecvError::Disconnected) => senders = false,
            }
        }
        session.fill();
        // Forward what the receiver has room for; discard if nobody is listening.
        loop {
            let permit = match events.try_reserve() {
                Ok(permit) => Some(permit),
                Err(mpsc::error::TrySendError::Full(())) => break,
                Err(mpsc::error::TrySendError::Closed(())) => None,
            };
            let Some((item, terminal)) = session.take() else {
                break;
            };
            match (permit, item) {
                (Some(permit), item) => permit.send(item),
                (None, Err(error)) if terminal => return Err(error),
                (None, _) => {},
            }
            if terminal {
                return Ok(());
            }
        }
        if !senders && events.is_closed() {
            return session.flush().await;
        }

        let accept = senders && session.can_queue();
        let forward = session.has_output() && !events.is_closed();
        let wake = tokio::select! {
            () = session.wait() => Wake::Ready,
            command = commands.recv(), if accept => Wake::Command(command),
            permit = events.reserve(), if forward => Wake::Permit(permit.ok()),
        };
        match wake {
            Wake::Ready | Wake::Permit(None) => {},
            Wake::Command(None) => senders = false,
            Wake::Command(Some(command)) => {
                if apply(&mut session, command).await {
                    return finish(session, &events).await;
                }
            },
            Wake::Permit(Some(permit)) => {
                if let Some((item, terminal)) = session.take() {
                    permit.send(item);
                    if terminal {
                        return Ok(());
                    }
                }
            },
        }
    }
}

/// Apply one command. Returns true after a shutdown.
async fn apply(session: &mut Session<Tokio>, command: Command) -> bool {
    match command {
        Command::Packet(packet, ack) => {
            let result = session.queue(packet);
            match ack {
                Some(ack) => {
                    let _ = ack.send(result);
                },
                None => {
                    if let Err(error) = result {
                        tracing::warn!(%error, "Dropping a packet from try_send");
                    }
                },
            }
        },
        Command::Tap(sink, filter) => session.insert_tap(sink, filter),
        Command::Request(pending) => {
            // Register only once queued, but report a queueing error to the requester.
            let (packet, sink, filter) = session.stamp(pending);
            match session.queue(packet) {
                Ok(()) => session.insert_tap(sink, filter),
                Err(error) => sink.fail(error),
            }
        },
        Command::Shutdown(ack) => {
            let _ = ack.send(session.shutdown().await);
            return true;
        },
    }
    false
}

/// Deliver what was read before a shutdown.
async fn finish(mut session: Session<Tokio>, events: &mpsc::Sender<Result<Packet>>) -> Result<()> {
    while let Some((item, _)) = session.take() {
        if events.send(item).await.is_err() {
            break;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;
    use crate::{
        identifiers::RequestId,
        insim::{Ism, Ncn, Tiny, TinyType},
        test::{Lfs, pair, pair_unstarted},
    };

    async fn connect() -> (Session<Tokio>, Lfs) {
        pair().await
    }
    async fn connect_with(ids: std::ops::RangeInclusive<u8>) -> (Session<Tokio>, Lfs) {
        let (unstarted, lfs) = pair_unstarted().await;
        (unstarted.with_request_ids(ids).resume(), lfs)
    }
    fn reqi(packet: &Packet) -> u8 {
        packet.request_id().unwrap().0
    }

    #[tokio::test]
    async fn split_requests_resolve_and_the_receiver_still_sees_every_reply() {
        tokio::time::timeout(Duration::from_secs(5), async {
            let (session, mut lfs) = connect_with(40..=41).await;
            let (sender, mut receiver, driver) = session.into_split(4);
            let driver = tokio::spawn(driver);

            let peer = async {
                let request = lfs.recv().await.unwrap();
                assert!(matches!(
                    request,
                    Packet::Tiny(Tiny {
                        subt: TinyType::Ism,
                        ..
                    })
                ));
                let id = reqi(&request);
                assert_eq!(id, 40);
                // An unrelated packet with the same type and another ReqI is not a reply.
                lfs.send(Ism {
                    reqi: RequestId(1),
                    ..Default::default()
                })
                .await;
                lfs.send(Ism {
                    reqi: RequestId(id),
                    hname: Some("host".into()),
                    ..Default::default()
                })
                .await;

                let request = lfs.recv().await.unwrap();
                let id = reqi(&request);
                for ucid in 0..3 {
                    lfs.send(Ncn {
                        reqi: RequestId(id),
                        ucid: ucid.into(),
                        ..Default::default()
                    })
                    .await;
                }
                lfs
            };
            let client = async {
                let ism: Ism = sender.request(TinyType::Ism).await.unwrap();
                assert_eq!(ism.hname.as_deref(), Some("host"));
                let ncns: Vec<Ncn> = sender
                    .request_all_within(TinyType::Ncn, Duration::from_millis(200))
                    .await
                    .unwrap();
                assert_eq!(ncns.len(), 3);
            };
            let (mut lfs, ()) = tokio::join!(peer, client);

            // Taps never take packets away from the receiver.
            let mut seen = Vec::new();
            for _ in 0..5 {
                seen.push(receiver.recv().await.unwrap().unwrap());
            }
            assert!(matches!(
                seen[0],
                Packet::Ism(Ism {
                    reqi: RequestId(1),
                    ..
                })
            ));
            assert!(matches!(seen[1], Packet::Ism(_)));
            assert!(seen[2..].iter().all(|p| matches!(p, Packet::Ncn(_))));

            sender.shutdown().await.unwrap();
            assert!(matches!(
                lfs.recv().await.unwrap(),
                Packet::Tiny(Tiny {
                    subt: TinyType::Close,
                    ..
                })
            ));
            assert!(receiver.recv().await.is_none());
            driver.await.unwrap().unwrap();
            assert!(sender.send(TinyType::Ping).await.is_err());
        })
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn driver_answers_keepalives_while_the_receiver_is_not_polled() {
        tokio::time::timeout(Duration::from_secs(5), async {
            let (session, mut lfs) = connect().await;
            let (sender, mut receiver, driver) = session.into_split(1);
            let driver = tokio::spawn(driver);
            // Fill the receiver's channel, then keep sending.
            for id in 1..=3 {
                lfs.send(Tiny {
                    reqi: RequestId(id),
                    subt: TinyType::Ping,
                })
                .await;
            }
            lfs.send(Tiny::default()).await;
            assert!(matches!(lfs.recv().await.unwrap(), Packet::Tiny(t) if t.is_keepalive()));
            // Everything is still delivered, in order.
            for id in [1, 2, 3, 0] {
                assert_eq!(reqi(&receiver.recv().await.unwrap().unwrap()), id);
            }
            // Dropping every handle stops the driver.
            drop((sender, receiver));
            driver.await.unwrap().unwrap();
        })
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn terminal_errors_reach_the_receiver_and_pending_requests() {
        tokio::time::timeout(Duration::from_secs(5), async {
            let (session, lfs) = connect().await;
            let (sender, mut receiver, driver) = session.into_split(4);
            let driver = tokio::spawn(driver);
            let request = sender.request::<Ism>(TinyType::Ism);
            let close = async {
                tokio::time::sleep(Duration::from_millis(50)).await;
                drop(lfs);
            };
            let (reply, ()) = tokio::join!(request, close);
            assert!(reply.is_err());
            // Disconnected or a reset, depending on whether the request was still unread.
            assert!(matches!(receiver.recv().await, Some(Err(_))));
            assert!(receiver.recv().await.is_none());
            driver.await.unwrap().unwrap();
        })
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn single_owner_request_fills_while_reading() {
        tokio::time::timeout(Duration::from_secs(5), async {
            let (mut session, mut lfs) = connect().await;
            let mut ism = session.request::<Ism>(TinyType::Ism).unwrap();
            let mut latest = session.tap(|p| matches!(p, Packet::Tiny(_)), Policy::Latest);
            session.flush().await.unwrap();
            let id = reqi(&lfs.recv().await.unwrap());
            lfs.send(Tiny {
                reqi: RequestId(1),
                subt: TinyType::Ping,
            })
            .await;
            lfs.send(Ism {
                reqi: RequestId(id),
                ..Default::default()
            })
            .await;
            let mut read = Vec::new();
            let reply = loop {
                tokio::select! {
                    packet = session.read() => read.push(packet.unwrap()),
                    reply = &mut ism => break reply.unwrap(),
                }
            };
            assert_eq!(reply.reqi, RequestId(id));
            assert!(ism.try_get().is_some_and(|r| r.is_err()));
            // The reply is still delivered by read().
            while read.len() < 2 {
                read.push(session.read().await.unwrap());
            }
            assert!(matches!(read[1], Packet::Ism(_)));
            assert_eq!(latest.try_recv().map(|p| reqi(&p)), Some(1));
            // A request nobody answers finishes after the quiet period.
            let mut none = session
                .request_all_within::<Ncn>(TinyType::Ncn, Duration::from_millis(50))
                .unwrap();
            let ncns = tokio::select! {
                packet = session.read() => panic!("unexpected {packet:?}"),
                ncns = &mut none => ncns.unwrap(),
            };
            assert!(ncns.is_empty());
        })
        .await
        .unwrap();
    }
}
