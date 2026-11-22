//! Keeping a [`World`] in step with a split session.

use insim::{
    Packet,
    net::{Policy, Sender},
};
use tokio::sync::mpsc;

use super::{SharedWorld, World, WorldEvent, WorldReady};

impl World {
    /// Send every [`STARTUP_REQUESTS`](Self::STARTUP_REQUESTS) at once and return
    /// all replies once each request has gone quiet. The replies are still read
    /// as ordinary packets; whoever folds them uses this to know when the world
    /// has caught up.
    pub async fn request_startup(sender: &Sender) -> insim::Result<Vec<Packet>> {
        let mut requests = Vec::with_capacity(Self::STARTUP_REQUESTS.len());
        for t in Self::STARTUP_REQUESTS {
            requests.push(sender.request_all::<Packet>(t.clone()));
        }
        let mut replies = Vec::new();
        for result in futures::future::join_all(requests).await {
            replies.extend(result?);
        }
        Ok(replies)
    }
}

impl SharedWorld {
    /// Keep this world in step with a split session from a background task, and
    /// report what changes.
    ///
    /// The task copies every InSim packet with a tap (nothing is taken from the
    /// session's receiver), sends the startup requests and anything else the world
    /// asks for ([`World::take_requests`]), and folds packets in the order they
    /// were read. The returned channel carries each
    /// resulting [`WorldEvent`], then [`WorldEvent::Ready`] once every startup
    /// reply has been folded. It closes when the session ends. Drain it or drop it.
    ///
    /// Folding runs alongside your own reads rather than in step with them: a
    /// packet you have just received may not be in the world yet. For handlers
    /// that need settled state, fold with [`World::apply_packet`] in your own
    /// loop instead (as kitcar does).
    ///
    /// Must be called within a Tokio runtime.
    pub async fn attach(
        &self,
        sender: &Sender,
    ) -> insim::Result<mpsc::UnboundedReceiver<WorldEvent>> {
        // Every InSim packet; telemetry has no ReqI and the world ignores it.
        let tap = sender
            .tap(|p| p.request_id().is_some(), Policy::Queue)
            .await?;
        let (events, receiver) = mpsc::unbounded_channel();
        let world = self.clone();
        let sender = sender.clone();
        drop(tokio::spawn(async move {
            let startup = World::request_startup(&sender);
            tokio::pin!(startup);
            let mut tap = tap;
            let mut starting = true;
            loop {
                tokio::select! {
                    packet = tap.recv() => match packet {
                        Some(packet) => world.fold(&packet, &sender, &events).await,
                        None => return,
                    },
                    replies = &mut startup, if starting => {
                        starting = false;
                        if let Err(error) = replies {
                            tracing::warn!(%error, "startup requests failed; WorldReady will not be emitted");
                            continue;
                        }
                        // A reply reaches the tap when it is decoded, before its
                        // request completes, so every reply is already queued.
                        while let Some(packet) = tap.try_recv() {
                            world.fold(&packet, &sender, &events).await;
                        }
                        if world.write().mark_ready() {
                            let _ = events.send(WorldEvent::Ready(WorldReady));
                        }
                    },
                }
            }
        }));
        Ok(receiver)
    }

    async fn fold(
        &self,
        packet: &Packet,
        sender: &Sender,
        events: &mpsc::UnboundedSender<WorldEvent>,
    ) {
        // Never hold the lock across an await.
        let (changes, requests) = {
            let mut world = self.write();
            (world.apply_packet(packet), world.take_requests())
        };
        for request in requests {
            if let Err(error) = sender.send(request).await {
                tracing::warn!(%error, "failed to send a request for the world");
            }
        }
        for event in changes {
            let _ = events.send(event);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use insim::{
        identifiers::ConnectionId,
        insim::{Ncn, Tiny, TinyType},
    };

    use super::*;

    #[tokio::test]
    async fn attach_folds_startup_replies_before_ready_without_taking_packets() {
        tokio::time::timeout(Duration::from_secs(10), async {
            let (session, mut lfs) = insim::test::pair().await;
            let (sender, mut receiver, driver) = session.into_split(16);
            let _driver = tokio::spawn(driver);
            let world = SharedWorld::default();
            let mut events = world.attach(&sender).await.unwrap();

            for _ in World::STARTUP_REQUESTS {
                let request: Tiny = lfs.expect().await;
                if request.subt == TinyType::Ncn {
                    lfs.send(Ncn {
                        reqi: request.reqi,
                        ucid: ConnectionId(3),
                        uname: "someone".into(),
                        ..Default::default()
                    })
                    .await;
                }
            }
            let mut connected = false;
            loop {
                match events.recv().await.unwrap() {
                    // A reply: the connection was there before us.
                    WorldEvent::Connected(c) => connected = c.existing,
                    WorldEvent::Ready(_) => break,
                    _ => {},
                }
            }
            assert!(connected, "the startup reply is folded before Ready");
            assert!(world.read().connection(ConnectionId(3)).is_some());
            assert!(world.read().is_ready());
            // The session's own receiver still gets the reply.
            assert!(matches!(receiver.recv().await, Some(Ok(Packet::Ncn(_)))));
        })
        .await
        .unwrap();
    }
}
