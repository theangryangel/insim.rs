//! The host end to end, against a fake LFS.

use insim::{
    Packet,
    identifiers::ConnectionId,
    insim::{Mtc, Tiny, TinyType},
    test::{Lfs, pair},
};

use super::*;
use crate::{
    kitcar::{
        Effects,
        service::{Context, Input},
    },
    world::WorldEvent,
};

#[derive(Debug, PartialEq)]
struct TickKey;

#[derive(Debug, Clone)]
enum Msg {
    Tick,
    Loaded(u32),
}

/// Announces every step it sees, so the fake LFS can follow along.
struct Probe(&'static str);

impl Service for Probe {
    type Msg = Msg;
    fn update(&mut self, input: Input<Msg>, cx: &Context<'_>) -> Effects<Msg> {
        let say = |text: String| Effects::new().message(text, ConnectionId::ALL);
        let name = self.0;
        match input {
            Input::Started => say(format!("{name} started")).schedule_after(
                TickKey,
                Duration::from_millis(20),
                Msg::Tick,
            ),
            Input::Msg(Msg::Tick) => say(format!("{name} tick")).task(async { Msg::Loaded(7) }),
            Input::Msg(Msg::Loaded(n)) => say(format!("{name} loaded {n}")),
            Input::World(WorldEvent::Ready(_)) => {
                say(format!("{name} ready {}", cx.world().is_ready()))
            },
            Input::Packet(Packet::Tiny(Tiny {
                subt: TinyType::Ping,
                ..
            })) => say(format!("{name} pong")).stop(),
            Input::Stopping => say(format!("{name} bye")),
            _ => Effects::none(),
        }
    }
}

/// The next chat line from the host, skipping its own startup requests.
async fn line(lfs: &mut Lfs) -> String {
    loop {
        match lfs.recv().await.expect("host closed the connection") {
            Packet::Mtc(Mtc { text, .. }) => return text,
            Packet::Tiny(Tiny { reqi, .. }) if reqi.0 != 0 => {},
            other => panic!("unexpected {other:?}"),
        }
    }
}

#[tokio::test]
async fn modes_get_timers_tasks_ready_and_stop_in_order() {
    tokio::time::timeout(Duration::from_secs(10), async {
        let (session, mut lfs) = pair().await;
        let host = Host::spawn(session);
        // The startup requests end with a ping.
        let ping = loop {
            if let Some(Packet::Tiny(Tiny {
                subt: TinyType::Ping,
                reqi,
            })) = lfs.recv().await
            {
                break reqi;
            }
        };
        let service = host.start(Probe("a")).await.unwrap();

        assert_eq!(line(&mut lfs).await, "a started");
        assert_eq!(line(&mut lfs).await, "a tick");
        assert_eq!(line(&mut lfs).await, "a loaded 7");
        // Its reply means every startup reply is in.
        lfs.send(Tiny {
            subt: TinyType::Reply,
            reqi: ping,
        })
        .await;
        assert_eq!(line(&mut lfs).await, "a ready true");

        service.stop().await;
        assert_eq!(line(&mut lfs).await, "a bye");

        // A new service on the same connection, without reconnecting.
        let mut service = host.start(Probe("b")).await.unwrap();
        assert_eq!(line(&mut lfs).await, "b started");
        assert_eq!(line(&mut lfs).await, "b tick");
        assert_eq!(line(&mut lfs).await, "b loaded 7");
        // A service can stop itself.
        lfs.send(TinyType::Ping).await;
        assert_eq!(line(&mut lfs).await, "b pong");
        assert_eq!(line(&mut lfs).await, "b bye");
        service.finished().await;

        host.shutdown().await.unwrap();
        let close: Tiny = lfs.expect().await;
        assert_eq!(close.subt, TinyType::Close);
    })
    .await
    .unwrap();
}

#[tokio::test]
async fn a_closed_connection_stops_modes_and_ends_the_host() {
    tokio::time::timeout(Duration::from_secs(10), async {
        let (session, lfs) = pair().await;
        let mut host = Host::spawn(session);
        let mut service = host.start(Probe("a")).await.unwrap();
        drop(lfs);
        service.finished().await;
        // The connection ended rather than being shut down.
        let _ = host.closed().await;
        assert!(matches!(host.closed().await, Err(HostError::Closed)));
        assert!(matches!(
            host.start(Probe("late")).await,
            Err(HostError::Closed)
        ));
    })
    .await
    .unwrap();
}

mod director;
mod ui;
