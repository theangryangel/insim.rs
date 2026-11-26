//! Views, components and clicks through the host.

use insim::insim::{Bfn, BfnType, Btc, Btn, BtnStyle, Ncn};

use super::*;
use crate::ui::{Child, Component, Node, Ui};

/// A self-contained dialog: opens on request, closes itself when clicked.
struct Dialog {
    open: bool,
}
#[derive(Clone, Debug)]
enum DialogMsg {
    Show,
    Close,
}
impl Component for Dialog {
    type Msg = DialogMsg;
    type Out = ();
    type Props<'a> = ();
    fn update(&mut self, msg: DialogMsg) -> crate::ui::Effects<DialogMsg> {
        self.open = matches!(msg, DialogMsg::Show);
        crate::ui::Effects::none()
    }
    fn render(&self, (): (), _: &mut Ui<'_, DialogMsg>) -> Node<DialogMsg> {
        if self.open {
            Node::clickable("help: close", BtnStyle::default(), DialogMsg::Close)
                .w(30)
                .h(5)
        } else {
            Node::empty()
        }
    }
}

#[derive(Clone, Debug)]
enum CounterMsg {
    Bump,
    Help(ConnectionId),
}

/// Shared state drawn straight into every player's view.
struct Counter {
    count: u32,
}
impl Service for Counter {
    type Msg = CounterMsg;
    fn update(&mut self, input: Input<CounterMsg>, _: &Context<'_>) -> Effects<CounterMsg> {
        match input {
            Input::Msg(CounterMsg::Bump) => {
                self.count += 1;
                Effects::none()
            },
            Input::Msg(CounterMsg::Help(ucid)) => {
                Effects::new().send(ucid, "help", DialogMsg::Show)
            },
            _ => Effects::none(),
        }
    }
    fn view(
        &self,
        ucid: ConnectionId,
        ui: &mut Ui<'_, CounterMsg>,
        _: &Context<'_>,
    ) -> Option<Node<CounterMsg>> {
        Some(
            Node::container().flex().flex_col().with_children([
                Node::clickable(
                    format!("count {}", self.count),
                    BtnStyle::default(),
                    CounterMsg::Bump,
                )
                .w(30)
                .h(5),
                Node::clickable("?", BtnStyle::default(), CounterMsg::Help(ucid))
                    .w(5)
                    .h(5),
                ui.child(Child::new(|| Dialog { open: false }).key("help")),
            ]),
        )
    }
}

/// The next button packet, skipping the host's startup requests.
async fn button(lfs: &mut Lfs) -> Packet {
    loop {
        match lfs.recv().await.expect("host closed the connection") {
            Packet::Tiny(Tiny { reqi, .. }) if reqi.0 != 0 => {},
            packet @ (Packet::Btn(_) | Packet::Bfn(_)) => return packet,
            other => panic!("unexpected {other:?}"),
        }
    }
}
async fn btn(lfs: &mut Lfs) -> Btn {
    match button(lfs).await {
        Packet::Btn(btn) => btn,
        other => panic!("expected a button, got {other:?}"),
    }
}

/// Loads its text as it mounts.
struct Loader {
    text: &'static str,
}
impl Component for Loader {
    type Msg = &'static str;
    type Out = ();
    type Props<'a> = ();
    fn mounted(&mut self, (): &()) -> crate::ui::Effects<&'static str> {
        crate::ui::Effects::new().task(async { "loaded" })
    }
    fn update(&mut self, text: &'static str) -> crate::ui::Effects<&'static str> {
        self.text = text;
        crate::ui::Effects::none()
    }
    fn render(&self, (): (), _: &mut Ui<'_, &'static str>) -> Node<&'static str> {
        Node::text(self.text, BtnStyle::default()).w(30).h(5)
    }
}

struct Loading;
impl Service for Loading {
    type Msg = ();
    fn update(&mut self, _: Input<()>, _: &Context<'_>) -> Effects<()> {
        Effects::none()
    }
    fn view(&self, _: ConnectionId, ui: &mut Ui<'_, ()>, _: &Context<'_>) -> Option<Node<()>> {
        Some(ui.component(|| Loader { text: "loading" }))
    }
}

#[tokio::test]
async fn components_tasks_report_back_through_the_host() {
    tokio::time::timeout(Duration::from_secs(10), async {
        let (session, mut lfs) = pair().await;
        let host = Host::spawn(session);
        let _service = host.start(Loading).await.unwrap();
        lfs.send(Ncn {
            ucid: ConnectionId(5),
            uname: "player".into(),
            ..Default::default()
        })
        .await;
        let loading = btn(&mut lfs).await;
        assert_eq!(loading.text, "loading");
        let loaded = btn(&mut lfs).await;
        assert_eq!(
            (loaded.clickid, loaded.text.as_str()),
            (loading.clickid, "loaded")
        );
        host.shutdown().await.unwrap();
    })
    .await
    .unwrap();
}

#[tokio::test]
async fn views_draw_state_route_clicks_and_clear_on_stop() {
    tokio::time::timeout(Duration::from_secs(10), async {
        let (session, mut lfs) = pair().await;
        let host = Host::spawn(session);
        let service = host.start(Counter { count: 0 }).await.unwrap();
        let player = ConnectionId(5);
        lfs.send(Ncn {
            ucid: player,
            uname: "player".into(),
            ..Default::default()
        })
        .await;

        let count = btn(&mut lfs).await;
        assert_eq!((count.ucid, count.text.as_str()), (player, "count 0"));
        let help = btn(&mut lfs).await;
        assert_eq!(help.text, "?");

        // The view's own button reaches the service, which redraws.
        lfs.send(Btc {
            ucid: player,
            clickid: count.clickid,
            ..Default::default()
        })
        .await;
        let redrawn = btn(&mut lfs).await;
        assert_eq!(
            (redrawn.clickid, redrawn.text.as_str()),
            (count.clickid, "count 1")
        );

        // The service opens the player's dialog by key...
        lfs.send(Btc {
            ucid: player,
            clickid: help.clickid,
            ..Default::default()
        })
        .await;
        let dialog = btn(&mut lfs).await;
        assert_eq!(dialog.text, "help: close");

        // ...and the dialog closes itself.
        lfs.send(Btc {
            ucid: player,
            clickid: dialog.clickid,
            ..Default::default()
        })
        .await;
        assert!(matches!(
            button(&mut lfs).await,
            Packet::Bfn(Bfn { subt: BfnType::DelBtn, clickid, .. }) if clickid == dialog.clickid
        ));

        // Stopping removes everything it drew.
        service.stop().await;
        let mut removed = Vec::new();
        for _ in 0..2 {
            match button(&mut lfs).await {
                Packet::Bfn(bfn) if bfn.subt == BfnType::DelBtn => {
                    removed.push(bfn.clickid);
                },
                other => panic!("expected a removal, got {other:?}"),
            }
        }
        removed.sort();
        let mut expected = vec![count.clickid, help.clickid];
        expected.sort();
        assert_eq!(removed, expected);
    })
    .await
    .unwrap();
}
