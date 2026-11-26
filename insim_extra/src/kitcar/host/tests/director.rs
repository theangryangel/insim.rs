//! Services reading and steering the director.

use insim::insim::{Ncn, RaceInProgress, Sta};

use super::*;
use crate::kitcar::director::{Combo, Cycle, DirectorCommand};

/// Reports what it reads of the director.
struct Watcher;

impl Service for Watcher {
    type Msg = ();
    fn update(&mut self, input: Input<()>, cx: &Context<'_>) -> Effects<()> {
        let say = |text: String| Effects::new().message(text, ConnectionId::ALL);
        match input {
            Input::Started => say(format!("watch {}", cx.director().is_some())),
            Input::Packet(Packet::Tiny(Tiny {
                subt: TinyType::Ping,
                ..
            })) => say(format!(
                "watch loading {:?}",
                cx.director().map(|director| director.loading)
            )),
            Input::World(WorldEvent::TrackChanged(_)) => say(format!(
                "watch ready {:?}",
                cx.director().map(|director| director.ready)
            )),
            _ => Effects::none(),
        }
    }
}

/// The next chat line from the watcher, skipping the director's own
/// commands and announcements.
async fn watched(lfs: &mut Lfs) -> String {
    loop {
        if let Packet::Mtc(Mtc { text, .. }) = lfs.recv().await.expect("host closed the connection")
            && text.starts_with("watch")
        {
            return text;
        }
    }
}

#[tokio::test]
async fn services_read_the_director_after_it_has_seen_each_input() {
    tokio::time::timeout(Duration::from_secs(10), async {
        let (session, mut lfs) = pair().await;
        let host = Host::spawn(session);
        let _watcher = host.start(Watcher).await.unwrap();
        assert_eq!(watched(&mut lfs).await, "watch false");

        let fe1x = "FE1X".parse().unwrap();
        host.start_director(RaceDirector::new(Combo::untimed(fe1x)))
            .await
            .unwrap();
        let _late = host.start(Watcher).await.unwrap();
        assert_eq!(watched(&mut lfs).await, "watch true");

        // Wait for the first combo to load.
        loop {
            lfs.send(TinyType::Ping).await;
            let both = [watched(&mut lfs).await, watched(&mut lfs).await];
            if both.iter().all(|line| line == "watch loading Some(false)") {
                break;
            }
        }

        // A player, then the lobby: the director sets it up, and both
        // watchers see it ready on the same input.
        lfs.send(Ncn {
            ucid: ConnectionId(1),
            uname: "driver".into(),
            ..Default::default()
        })
        .await;
        lfs.send(Sta {
            track: fe1x,
            raceinprog: RaceInProgress::No,
            ..Default::default()
        })
        .await;
        assert_eq!(watched(&mut lfs).await, "watch ready Some(true)");
        assert_eq!(watched(&mut lfs).await, "watch ready Some(true)");

        host.shutdown().await.unwrap();
    })
    .await
    .unwrap();
}

/// Asks the director to rotate when pinged.
struct Rotator;

impl Service for Rotator {
    type Msg = ();
    fn update(&mut self, input: Input<()>, _: &Context<'_>) -> Effects<()> {
        match input {
            Input::Packet(Packet::Tiny(Tiny {
                subt: TinyType::Ping,
                ..
            })) => Effects::new().director(DirectorCommand::Rotate),
            _ => Effects::none(),
        }
    }
}

#[tokio::test]
async fn services_steer_the_director() {
    tokio::time::timeout(Duration::from_secs(10), async {
        let (session, mut lfs) = pair().await;
        let host = Host::spawn(session);
        let combo = |track: &str| Combo::untimed(track.parse().unwrap());
        host.start_director(RaceDirector::new(Cycle::new([combo("FE1"), combo("BL1")])))
            .await
            .unwrap();
        let _rotator = host.start(Rotator).await.unwrap();

        lfs.send(TinyType::Ping).await;
        loop {
            if let Packet::Mtc(Mtc { text, .. }) =
                lfs.recv().await.expect("host closed the connection")
                && text.contains("Next track")
            {
                assert_eq!(text, "^3Next track: ^7BL1");
                break;
            }
        }
        host.shutdown().await.unwrap();
    })
    .await
    .unwrap();
}

#[tokio::test]
async fn a_stopped_director_leaves_services_without_one() {
    tokio::time::timeout(Duration::from_secs(10), async {
        let (session, mut lfs) = pair().await;
        let host = Host::spawn(session);
        let fe1x = "FE1X".parse().unwrap();
        host.start_director(RaceDirector::new(Combo::untimed(fe1x)))
            .await
            .unwrap();
        let _watcher = host.start(Watcher).await.unwrap();
        assert_eq!(watched(&mut lfs).await, "watch true");

        host.stop_director().await.unwrap();
        lfs.send(TinyType::Ping).await;
        assert_eq!(watched(&mut lfs).await, "watch loading None");
        // Stopping again does nothing.
        host.stop_director().await.unwrap();

        host.shutdown().await.unwrap();
    })
    .await
    .unwrap();
}
