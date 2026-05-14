//! In-crate unit tests for the dispatch pipeline.
//!
//! These exercise the runtime *without* a network connection by driving
//! [`crate::app::dispatch_cycle`] directly with hand-built `Dispatch` values.

// dev-deps used by examples but not by these tests; silence unused-crate lint.
use std::{
    str::FromStr,
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
};

use clap as _;
use fixedbitset as _;
use insim::{
    identifiers::ConnectionId,
    insim::{Mso, MsoUserType, Ncn},
};
use taffy as _;
use tokio::sync::mpsc;
use tracing_subscriber as _;

use crate::{
    App, AppError, ChatParser, Connected, Dispatch, Event, Packet, Presence, Sender, Spawned,
    State, app::dispatch_cycle, event::Command,
};

/// A toy typed chat enum for the parser test. In production this would be
/// `#[derive(insim_extras::chat::Parse)]` with a `FromStr` bridge.
#[derive(Debug, Clone, PartialEq, Eq)]
enum TestCmd {
    Ping { who: String },
}

impl FromStr for TestCmd {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let body = s.strip_prefix('!').ok_or(())?;
        let (head, rest) = body.split_once(char::is_whitespace).unwrap_or((body, ""));
        match head {
            "ping" => Ok(TestCmd::Ping {
                who: rest.trim().to_string(),
            }),
            _ => Err(()),
        }
    }
}

#[derive(Clone, Default)]
struct TestState {
    ncn_hits: Arc<AtomicUsize>,
    mso_hits: Arc<AtomicUsize>,
    connected_hits: Arc<AtomicUsize>,
    cmd_hits: Arc<AtomicUsize>,
    last_cmd: Arc<Mutex<Option<TestCmd>>>,
}

async fn count_ncn(Packet(_n): Packet<Ncn>, State(s): State<TestState>) -> Result<(), AppError> {
    let _ = s.ncn_hits.fetch_add(1, Ordering::Relaxed);
    Ok(())
}

async fn count_mso(Packet(_m): Packet<Mso>, State(s): State<TestState>) -> Result<(), AppError> {
    let _ = s.mso_hits.fetch_add(1, Ordering::Relaxed);
    Ok(())
}

async fn count_connected(
    Event(_c): Event<Connected>,
    State(s): State<TestState>,
) -> Result<(), AppError> {
    let _ = s.connected_hits.fetch_add(1, Ordering::Relaxed);
    Ok(())
}

async fn capture_cmd(
    Event(cmd): Event<TestCmd>,
    State(s): State<TestState>,
) -> Result<(), AppError> {
    let _ = s.cmd_hits.fetch_add(1, Ordering::Relaxed);
    *s.last_cmd.lock().expect("poison") = Some(cmd);
    Ok(())
}

fn make_ncn(ucid: u8, uname: &str) -> insim::Packet {
    insim::Packet::Ncn(Ncn {
        ucid: ConnectionId(ucid),
        uname: uname.to_string(),
        pname: uname.to_string(),
        admin: false,
        total: 1,
        ..Default::default()
    })
}

fn make_mso(ucid: u8, msg: &str) -> insim::Packet {
    insim::Packet::Mso(Mso {
        ucid: ConnectionId(ucid),
        msg: msg.to_string(),
        usertype: MsoUserType::User,
        textstart: 0,
        ..Default::default()
    })
}

fn app_with(state: TestState) -> App<TestState> {
    App::new()
        .with_state(state)
        .extension(Presence::new())
        .extension(ChatParser::<TestCmd>::new())
        .handler(count_ncn)
        .handler(count_mso)
        .handler(count_connected)
        .handler(capture_cmd)
}

/// Pull an app apart and drive one dispatch directly. Drains any synthetic
/// events emitted by extensions, simulating the main loop's behaviour of
/// cycling queued events through fresh dispatch_cycles.
async fn drive(app: App<TestState>, state: &TestState, d: Dispatch) {
    let (cmd_tx, mut cmd_rx) = mpsc::unbounded_channel::<Command>();
    let sender = Sender::new(cmd_tx);
    let cancel = tokio_util::sync::CancellationToken::new();
    let chain = app.extension_chain;
    let handlers = app.handlers;
    let extensions = app.extensions;

    dispatch_cycle(d, state, &sender, &chain, &handlers, &extensions, &cancel).await;

    // Drain anything middleware emitted while we were running.
    while let Ok(cmd) = cmd_rx.try_recv() {
        if let Command::Event(payload) = cmd {
            dispatch_cycle(
                Dispatch::Synthetic(payload),
                state,
                &sender,
                &chain,
                &handlers,
                &extensions,
                &cancel,
            )
            .await;
        }
        // Command::Packet is dropped - no wire to write to in tests.
    }
}

#[tokio::test]
async fn ncn_routes_to_packet_handler_and_synthetic_connected() {
    let state = TestState::default();
    drive(
        app_with(state.clone()),
        &state,
        Dispatch::Packet(make_ncn(7, "bob")),
    )
    .await;

    // Packet<Ncn> handler fired exactly once.
    assert_eq!(state.ncn_hits.load(Ordering::Relaxed), 1);
    // Presence emitted Connected, so Event<Connected> fired too.
    assert_eq!(state.connected_hits.load(Ordering::Relaxed), 1);
    // Mso handler did NOT fire (different routing key).
    assert_eq!(state.mso_hits.load(Ordering::Relaxed), 0);
}

#[tokio::test]
async fn mso_without_prefix_routes_only_to_packet_handler() {
    let state = TestState::default();
    drive(
        app_with(state.clone()),
        &state,
        Dispatch::Packet(make_mso(3, "hello world")),
    )
    .await;

    assert_eq!(state.mso_hits.load(Ordering::Relaxed), 1);
    assert_eq!(state.ncn_hits.load(Ordering::Relaxed), 0);
    // No prefix → no typed synthetic event.
    assert_eq!(state.cmd_hits.load(Ordering::Relaxed), 0);
}

#[tokio::test]
async fn chat_parser_emits_typed_event_for_prefixed_mso() {
    let state = TestState::default();
    drive(
        app_with(state.clone()),
        &state,
        Dispatch::Packet(make_mso(3, "!ping karl")),
    )
    .await;

    // The Mso handler still fires for the wire packet.
    assert_eq!(state.mso_hits.load(Ordering::Relaxed), 1);
    // And the typed synthetic event reaches Event<TestCmd>.
    assert_eq!(state.cmd_hits.load(Ordering::Relaxed), 1);
    let captured = state
        .last_cmd
        .lock()
        .expect("poison")
        .clone()
        .expect("command captured");
    assert_eq!(
        captured,
        TestCmd::Ping {
            who: "karl".to_string(),
        }
    );
}

#[tokio::test]
async fn unrelated_packet_fires_no_handlers() {
    let state = TestState::default();
    // A packet variant with no registered handler.
    let packet = insim::Packet::Tiny(insim::insim::Tiny::default());
    drive(app_with(state.clone()), &state, Dispatch::Packet(packet)).await;

    assert_eq!(state.ncn_hits.load(Ordering::Relaxed), 0);
    assert_eq!(state.mso_hits.load(Ordering::Relaxed), 0);
    assert_eq!(state.connected_hits.load(Ordering::Relaxed), 0);
    assert_eq!(state.cmd_hits.load(Ordering::Relaxed), 0);
}

#[tokio::test]
async fn presence_is_queryable_via_extractor() {
    // The extractor-driven handler reads the live connection map and stashes
    // the count it saw at the moment of dispatch. Registering `presence` as an
    // extension wires both the event observer and the extractor source.
    #[derive(Clone, Default)]
    struct PState {
        last_seen_count: Arc<AtomicUsize>,
    }

    async fn observe_count(presence: Presence, State(s): State<PState>) -> Result<(), AppError> {
        s.last_seen_count.store(presence.count(), Ordering::Relaxed);
        Ok(())
    }

    let state = PState::default();
    let presence = Presence::new();
    let app: App<PState> = App::new()
        .with_state(state.clone())
        .extension(presence.clone())
        .handler(observe_count);

    let (cmd_tx, _cmd_rx) = mpsc::unbounded_channel::<Command>();
    let sender = Sender::new(cmd_tx);
    let cancel = tokio_util::sync::CancellationToken::new();
    let chain = app.extension_chain;
    let handlers = app.handlers;
    let extensions = app.extensions;

    // First NCN: presence inserts; handler reads count = 1 after middleware ran.
    dispatch_cycle(
        Dispatch::Packet(make_ncn(1, "alice")),
        &state,
        &sender,
        &chain,
        &handlers,
        &extensions,
        &cancel,
    )
    .await;
    assert_eq!(state.last_seen_count.load(Ordering::Relaxed), 1);

    // Second NCN: count rises to 2.
    dispatch_cycle(
        Dispatch::Packet(make_ncn(2, "bob")),
        &state,
        &sender,
        &chain,
        &handlers,
        &extensions,
        &cancel,
    )
    .await;
    assert_eq!(state.last_seen_count.load(Ordering::Relaxed), 2);

    // External read via the same `presence` handle reflects the live state.
    assert_eq!(presence.count(), 2);
    assert!(presence.get(ConnectionId(1)).is_some());
    assert!(presence.get(ConnectionId(2)).is_some());
}

#[tokio::test]
async fn spawned_handler_spawns_once_and_forwards_dispatches() {
    // `Spawned` should spawn its async fn exactly once (on first dispatch) and
    // forward every dispatch - including the very first - into the channel
    // the spawned task drains.
    let spawn_count = Arc::new(AtomicUsize::new(0));
    let dispatch_count = Arc::new(AtomicUsize::new(0));

    let spawned = Spawned::new({
        let spawn_count = spawn_count.clone();
        let dispatch_count = dispatch_count.clone();
        move |mut rx: mpsc::UnboundedReceiver<Dispatch>, _sender: Sender| async move {
            let _ = spawn_count.fetch_add(1, Ordering::Relaxed);
            while let Some(_d) = rx.recv().await {
                let _ = dispatch_count.fetch_add(1, Ordering::Relaxed);
            }
        }
    });

    let app: App<()> = App::new().with_state(()).handler(spawned);

    let (cmd_tx, _cmd_rx) = mpsc::unbounded_channel::<Command>();
    let sender = Sender::new(cmd_tx);
    let cancel = tokio_util::sync::CancellationToken::new();
    let state = ();
    let chain = app.extension_chain;
    let handlers = app.handlers;
    let extensions = app.extensions;

    dispatch_cycle(
        Dispatch::Packet(make_ncn(1, "alice")),
        &state,
        &sender,
        &chain,
        &handlers,
        &extensions,
        &cancel,
    )
    .await;
    dispatch_cycle(
        Dispatch::Packet(make_ncn(2, "bob")),
        &state,
        &sender,
        &chain,
        &handlers,
        &extensions,
        &cancel,
    )
    .await;

    // Give the spawned task a moment to drain the buffered dispatches.
    tokio::time::sleep(std::time::Duration::from_millis(20)).await;

    assert_eq!(
        spawn_count.load(Ordering::Relaxed),
        1,
        "spawned task should run exactly once"
    );
    assert_eq!(
        dispatch_count.load(Ordering::Relaxed),
        2,
        "both dispatches should be forwarded"
    );
}

#[tokio::test]
async fn sender_event_pushes_into_back_channel() {
    // Confirms that Sender::event reaches the dispatcher's back-channel - the
    // serve loop drains this channel on its own; here we just verify the
    // command shows up.
    let (cmd_tx, mut cmd_rx) = mpsc::unbounded_channel::<Command>();
    let sender = Sender::new(cmd_tx);

    sender.event("hello".to_string()).expect("event ok");
    let cmd = cmd_rx.recv().await.expect("got command");
    match cmd {
        Command::Event(payload) => {
            let s = payload
                .downcast_ref::<String>()
                .expect("payload is String")
                .clone();
            assert_eq!(s, "hello");
        },
        _ => panic!("expected Command::Event"),
    }
}
