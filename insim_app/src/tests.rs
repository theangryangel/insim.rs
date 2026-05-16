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
    App, AppError, ChatParser, Connected, Dispatch, Event, Game, HandlerExt, Packet, Presence,
    RaceStarted, Sender, Spawned, State, TrackChanged, always, app::dispatch_cycle, event::Command,
    in_state, never,
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
    // Presence needs a Sender for its admin commands; tests don't exercise
    // those, so feed it a sender whose receiver is dropped immediately.
    let (cmd_tx, _) = mpsc::unbounded_channel::<Command>();
    let dummy_sender = Sender::new(cmd_tx);
    App::new()
        .with_state(state)
        .extension(Presence::new(dummy_sender))
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
    let (cmd_tx, _cmd_rx) = mpsc::unbounded_channel::<Command>();
    let sender = Sender::new(cmd_tx);
    let presence = Presence::new(sender.clone());
    let app: App<PState> = App::new()
        .with_state(state.clone())
        .extension(presence.clone())
        .handler(observe_count);

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
async fn cancellation_token_extractor_triggers_shutdown() {
    // A handler that pulls `CancellationToken` via FromContext and calls
    // `.cancel()` brings the runtime's token down. This is the round-trip
    // equivalent of a `!quit` chat command.
    async fn quit_on_ncn(
        _: Packet<Ncn>,
        cancel: tokio_util::sync::CancellationToken,
    ) -> Result<(), AppError> {
        cancel.cancel();
        Ok(())
    }

    let app: App<()> = App::new().with_state(()).handler(quit_on_ncn);
    let (cmd_tx, _cmd_rx) = mpsc::unbounded_channel::<Command>();
    let sender = Sender::new(cmd_tx);
    let cancel = tokio_util::sync::CancellationToken::new();
    let state = ();
    let chain = app.extension_chain;
    let handlers = app.handlers;
    let extensions = app.extensions;

    assert!(!cancel.is_cancelled());
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
    assert!(
        cancel.is_cancelled(),
        "handler should have cancelled the runtime token"
    );
}

#[tokio::test]
async fn spawned_handler_channel_closes_on_cancel() {
    // When the runtime's cancel token fires, Spawned's internal forwarder
    // drops the user-facing tx, so a task whose body is just
    // `while let Some(_) = rx.recv().await {}` exits without needing an
    // explicit cancel check of its own.
    let task_exited = Arc::new(AtomicUsize::new(0));

    let spawned = Spawned::new({
        let task_exited = task_exited.clone();
        move |mut rx: mpsc::UnboundedReceiver<Dispatch>, _sender: Sender| async move {
            while rx.recv().await.is_some() {}
            let _ = task_exited.fetch_add(1, Ordering::Relaxed);
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

    // First dispatch spawns the user task + forwarder.
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

    // Sanity: task is alive, blocked on rx.recv().
    tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    assert_eq!(
        task_exited.load(Ordering::Relaxed),
        0,
        "task should still be running before cancel"
    );

    // Trigger cancel - forwarder drops user_tx, user rx returns None.
    cancel.cancel();
    tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    assert_eq!(
        task_exited.load(Ordering::Relaxed),
        1,
        "task should have exited after cancel closed the channel"
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

// ---------------------------------------------------------------------------
// run_if + transition-event tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn run_if_skips_handler_when_predicate_false() {
    let state = TestState::default();
    let app: App<TestState> = App::new()
        .with_state(state.clone())
        .handler(count_ncn.run_if(never()));

    let (cmd_tx, _cmd_rx) = mpsc::unbounded_channel::<Command>();
    let sender = Sender::new(cmd_tx);
    let cancel = tokio_util::sync::CancellationToken::new();
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

    assert_eq!(
        state.ncn_hits.load(Ordering::Relaxed),
        0,
        "handler should be gated off by never()"
    );
}

#[tokio::test]
async fn run_if_runs_handler_when_predicate_true() {
    let state = TestState::default();
    let app: App<TestState> = App::new()
        .with_state(state.clone())
        .handler(count_ncn.run_if(always()));

    let (cmd_tx, _cmd_rx) = mpsc::unbounded_channel::<Command>();
    let sender = Sender::new(cmd_tx);
    let cancel = tokio_util::sync::CancellationToken::new();
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

    assert_eq!(
        state.ncn_hits.load(Ordering::Relaxed),
        1,
        "handler should fire under always()"
    );
}

#[tokio::test]
async fn in_state_reads_extension_and_gates_handler() {
    // An extension that just holds a boolean. The handler is gated on it
    // through `in_state`. Flipping the boolean between dispatches changes
    // whether the handler fires.
    use std::sync::RwLock;

    use crate::{Extension, ExtractCx, FromContext};

    #[derive(Clone)]
    struct Flag(Arc<RwLock<bool>>);

    impl<S: Send + Sync + 'static> Extension<S> for Flag {}

    impl<S: Send + Sync + 'static> FromContext<S> for Flag {
        fn from_context(cx: &ExtractCx<'_, S>) -> Option<Self> {
            cx.extensions.get::<Flag>()
        }
    }

    let state = TestState::default();
    let flag = Flag(Arc::new(RwLock::new(false)));
    let app: App<TestState> = App::new()
        .with_state(state.clone())
        .extension(flag.clone())
        .handler(count_ncn.run_if(in_state(|f: &Flag| *f.0.read().expect("poison"))));

    let (cmd_tx, _cmd_rx) = mpsc::unbounded_channel::<Command>();
    let sender = Sender::new(cmd_tx);
    let cancel = tokio_util::sync::CancellationToken::new();
    let chain = app.extension_chain;
    let handlers = app.handlers;
    let extensions = app.extensions;

    // flag = false: handler skipped.
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
    assert_eq!(
        state.ncn_hits.load(Ordering::Relaxed),
        0,
        "should be gated off while flag=false"
    );

    // Flip the flag - the same handler now passes the predicate.
    *flag.0.write().expect("poison") = true;

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
    assert_eq!(
        state.ncn_hits.load(Ordering::Relaxed),
        1,
        "handler should fire once flag flipped"
    );
}

#[tokio::test]
async fn game_emits_race_started_on_sta_transition() {
    use insim::insim::{RaceInProgress, Sta};

    let (cmd_tx, mut cmd_rx) = mpsc::unbounded_channel::<Command>();
    let sender = Sender::new(cmd_tx);

    // Game::new() fires an Sst request - drop any commands it queued.
    let game = Game::new(sender.clone());
    while cmd_rx.try_recv().is_ok() {}

    let app: App<()> = App::new().with_state(()).extension(game.clone());

    let cancel = tokio_util::sync::CancellationToken::new();
    let chain = app.extension_chain;
    let handlers = app.handlers;
    let extensions = app.extensions;
    let state = ();

    // First Sta with racing=No: matches initial state, no RaceStarted.
    dispatch_cycle(
        Dispatch::Packet(insim::Packet::Sta(Sta {
            raceinprog: RaceInProgress::No,
            ..Default::default()
        })),
        &state,
        &sender,
        &chain,
        &handlers,
        &extensions,
        &cancel,
    )
    .await;

    let mut events = Vec::new();
    while let Ok(cmd) = cmd_rx.try_recv() {
        if let Command::Event(payload) = cmd {
            events.push(payload);
        }
    }
    assert!(
        !events.iter().any(|p| p.is::<RaceStarted>()),
        "no RaceStarted expected on No → No"
    );

    // Second Sta with racing=Racing: transition fires RaceStarted.
    dispatch_cycle(
        Dispatch::Packet(insim::Packet::Sta(Sta {
            raceinprog: RaceInProgress::Racing,
            ..Default::default()
        })),
        &state,
        &sender,
        &chain,
        &handlers,
        &extensions,
        &cancel,
    )
    .await;

    let mut events = Vec::new();
    while let Ok(cmd) = cmd_rx.try_recv() {
        if let Command::Event(payload) = cmd {
            events.push(payload);
        }
    }
    let started = events.iter().filter(|p| p.is::<RaceStarted>()).count();
    assert_eq!(
        started, 1,
        "RaceStarted should fire exactly once on transition to Racing"
    );
}

#[tokio::test]
async fn periodic_emits_events_on_schedule() {
    use std::time::Duration;

    #[derive(Clone, Debug)]
    struct Tick;

    // Build an App with a fast periodic ticker. Take cmd_rx out and drain it
    // after a short wait; we should see several Ticks.
    let mut app: App<()> = App::new()
        .with_state(())
        .periodic(Duration::from_millis(5), Tick);

    let mut cmd_rx = app
        .cmd_rx
        .take()
        .expect("cmd_rx present before serve consumes it");

    tokio::time::sleep(Duration::from_millis(40)).await;

    let mut tick_count = 0usize;
    while let Ok(cmd) = cmd_rx.try_recv() {
        if let Command::Event(payload) = cmd
            && payload.is::<Tick>()
        {
            tick_count += 1;
        }
    }

    assert!(
        tick_count >= 3,
        "expected at least 3 ticks in 40ms, got {tick_count}"
    );

    // Cancel so the spawned task winds down before the runtime drops.
    app.cancel.cancel();
}

#[tokio::test]
async fn game_emits_track_changed_on_track_field_change() {
    use insim::core::track::Track;
    use insim::insim::Sta;

    let (cmd_tx, mut cmd_rx) = mpsc::unbounded_channel::<Command>();
    let sender = Sender::new(cmd_tx);

    let game = Game::new(sender.clone());
    while cmd_rx.try_recv().is_ok() {}

    let app: App<()> = App::new().with_state(()).extension(game.clone());

    let cancel = tokio_util::sync::CancellationToken::new();
    let chain = app.extension_chain;
    let handlers = app.handlers;
    let extensions = app.extensions;
    let state = ();

    let track_a = Track::ALL[0];
    let track_b = *Track::ALL
        .iter()
        .find(|t| **t != track_a)
        .expect("at least two tracks");

    // First Sta with track_a: prev was None, fires TrackChanged { from: None, to: track_a }.
    dispatch_cycle(
        Dispatch::Packet(insim::Packet::Sta(Sta {
            track: track_a,
            ..Default::default()
        })),
        &state,
        &sender,
        &chain,
        &handlers,
        &extensions,
        &cancel,
    )
    .await;

    let mut events = Vec::new();
    while let Ok(cmd) = cmd_rx.try_recv() {
        if let Command::Event(payload) = cmd {
            events.push(payload);
        }
    }
    let changes: Vec<_> = events
        .iter()
        .filter_map(|p| p.downcast_ref::<TrackChanged>())
        .collect();
    assert_eq!(changes.len(), 1, "first Sta should emit TrackChanged");
    assert_eq!(changes[0].from, None);
    assert_eq!(changes[0].to, track_a);

    // Same track again - no event.
    dispatch_cycle(
        Dispatch::Packet(insim::Packet::Sta(Sta {
            track: track_a,
            ..Default::default()
        })),
        &state,
        &sender,
        &chain,
        &handlers,
        &extensions,
        &cancel,
    )
    .await;

    let mut events = Vec::new();
    while let Ok(cmd) = cmd_rx.try_recv() {
        if let Command::Event(payload) = cmd {
            events.push(payload);
        }
    }
    assert!(
        !events.iter().any(|p| p.is::<TrackChanged>()),
        "same track should not emit TrackChanged"
    );

    // Different track - emits with from = Some(track_a), to = track_b.
    dispatch_cycle(
        Dispatch::Packet(insim::Packet::Sta(Sta {
            track: track_b,
            ..Default::default()
        })),
        &state,
        &sender,
        &chain,
        &handlers,
        &extensions,
        &cancel,
    )
    .await;

    let mut events = Vec::new();
    while let Ok(cmd) = cmd_rx.try_recv() {
        if let Command::Event(payload) = cmd {
            events.push(payload);
        }
    }
    let changes: Vec<_> = events
        .iter()
        .filter_map(|p| p.downcast_ref::<TrackChanged>())
        .collect();
    assert_eq!(changes.len(), 1, "track change should emit TrackChanged");
    assert_eq!(changes[0].from, Some(track_a));
    assert_eq!(changes[0].to, track_b);
}
