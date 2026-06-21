//! In-crate unit tests for the dispatch pipeline.
//!
//! These exercise the runtime *without* a network connection by driving
//! [`crate::app::dispatch_cycle`] directly with hand-built `Dispatch` values.

// Tests run on `App<()>`, so the `let app_state = app.state;` destructuring
// (kept uniform with `run()`) binds the unit type; allow it crate-test-wide
// rather than special-casing every test body.
#![allow(clippy::let_unit_value)]
// Test `Handler::call` impls are written long-hand to carry the trait's
// `impl Future + Send` bound (an `async fn` desugaring would drop `Send`),
// same as the real impls; silence the false-positive suggestion.
#![allow(clippy::manual_async_fn)]

// dev-deps used by examples but not by these tests; silence unused-crate lint.
use std::{
    str::FromStr,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};

use clap as _;
use insim::{
    identifiers::ConnectionId,
    insim::{Mso, MsoUserType, Ncn},
};
use parking_lot::{Mutex, RwLock};
use tokio::sync::mpsc;
use tracing_subscriber as _;

use super::{event::Command, runtime::dispatch_cycle};
use crate::{
    App, AppError, ChatEvent, ChatParser, Connected, Dispatch, Event, ExtractCx, Handler,
    HandlerExt, Packet, Sender, Stage, State, Svc, World,
};

/// A toy typed chat enum for the parser test.
#[derive(Debug, Clone, PartialEq, Eq)]
enum TestCmd {
    Ping { who: String },
}

impl FromStr for TestCmd {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (head, rest) = s.split_once(char::is_whitespace).unwrap_or((s, ""));
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

impl<S: Send + Sync + 'static> Handler<(), S> for TestState {}

async fn count_ncn(Packet(_n): Packet<Ncn>, Svc(s): Svc<TestState>) -> Result<(), AppError> {
    let _ = s.ncn_hits.fetch_add(1, Ordering::Relaxed);
    Ok(())
}

async fn count_mso(Packet(_m): Packet<Mso>, Svc(s): Svc<TestState>) -> Result<(), AppError> {
    let _ = s.mso_hits.fetch_add(1, Ordering::Relaxed);
    Ok(())
}

async fn count_connected(
    Event(_c): Event<Connected>,
    Svc(s): Svc<TestState>,
) -> Result<(), AppError> {
    let _ = s.connected_hits.fetch_add(1, Ordering::Relaxed);
    Ok(())
}

async fn capture_cmd(
    Event(cmd): Event<ChatEvent<TestCmd>>,
    Svc(s): Svc<TestState>,
) -> Result<(), AppError> {
    let _ = s.cmd_hits.fetch_add(1, Ordering::Relaxed);
    *s.last_cmd.lock() = Some(cmd.parsed);
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

fn app_with(state: TestState) -> App {
    App::new()
        .handle(Stage::Update, state)
        .handle(Stage::Update, ChatParser::<TestCmd>::new(&['!', '?']))
        .handle(Stage::Update, count_ncn)
        .handle(Stage::Update, count_mso)
        .handle(Stage::Update, count_connected)
        .handle(Stage::Update, capture_cmd)
}

/// Pull an app apart and drive one dispatch directly, mirroring the runtime
/// loop. `dispatch_cycle` folds packets into the intrinsic world and dispatches
/// the derived world events synchronously; we then drain any *user* events
/// handlers emitted via the back-channel (e.g. `ChatParser`'s typed events) and
/// cycle those too.
async fn drive(app: App, d: Dispatch) {
    let (cmd_tx, mut cmd_rx) = mpsc::unbounded_channel::<Command>();
    let sender = Sender::new(cmd_tx);
    let cancel = tokio_util::sync::CancellationToken::new();
    let app_state = app.state;
    let world = app.world;
    let pre_handlers = app.pre_handlers;
    let update_handlers = app.update_handlers;

    dispatch_cycle(
        d,
        &sender,
        &world,
        None,
        &app_state,
        &pre_handlers,
        &update_handlers,
        &cancel,
    )
    .await;

    // Drain anything middleware emitted while we were running.
    while let Ok(cmd) = cmd_rx.try_recv() {
        if let Command::Event(payload) = cmd {
            dispatch_cycle(
                Dispatch::Synthetic(payload),
                &sender,
                &world,
                None,
                &app_state,
                &pre_handlers,
                &update_handlers,
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
        Dispatch::Packet(make_mso(3, "hello world")),
    )
    .await;

    assert_eq!(state.mso_hits.load(Ordering::Relaxed), 1);
    assert_eq!(state.ncn_hits.load(Ordering::Relaxed), 0);
    // No prefix -> no typed synthetic event.
    assert_eq!(state.cmd_hits.load(Ordering::Relaxed), 0);
}

#[tokio::test]
async fn chat_parser_emits_typed_event_for_prefixed_mso() {
    let state = TestState::default();
    drive(
        app_with(state.clone()),
        Dispatch::Packet(make_mso(3, "!ping karl")),
    )
    .await;

    // The Mso handler still fires for the wire packet.
    assert_eq!(state.mso_hits.load(Ordering::Relaxed), 1);
    // And the typed synthetic event reaches Event<TestCmd>.
    assert_eq!(state.cmd_hits.load(Ordering::Relaxed), 1);
    let captured = state.last_cmd.lock().clone().expect("command captured");
    assert_eq!(
        captured,
        TestCmd::Ping {
            who: "karl".to_string(),
        }
    );
}

#[tokio::test]
async fn chat_parser_emits_typed_event_for_alt_prefixed_mso() {
    let state = TestState::default();
    drive(
        app_with(state.clone()),
        Dispatch::Packet(make_mso(3, "?ping karl")),
    )
    .await;

    // The Mso handler still fires for the wire packet.
    assert_eq!(state.mso_hits.load(Ordering::Relaxed), 1);
    // And the typed synthetic event reaches Event<TestCmd>.
    assert_eq!(state.cmd_hits.load(Ordering::Relaxed), 1);
    let captured = state.last_cmd.lock().clone().expect("command captured");
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
    let packet = insim::Packet::Tiny(insim::insim::Tiny::default());
    drive(app_with(state.clone()), Dispatch::Packet(packet)).await;

    assert_eq!(state.ncn_hits.load(Ordering::Relaxed), 0);
    assert_eq!(state.mso_hits.load(Ordering::Relaxed), 0);
    assert_eq!(state.connected_hits.load(Ordering::Relaxed), 0);
    assert_eq!(state.cmd_hits.load(Ordering::Relaxed), 0);
}

#[tokio::test]
async fn presence_is_queryable_via_extractor() {
    // The extractor-driven handler reads the live connection map and stashes
    // the count it saw at the moment of dispatch.
    #[derive(Clone, Default)]
    struct PState {
        last_seen_count: Arc<AtomicUsize>,
    }

    impl<S: Send + Sync + 'static> Handler<(), S> for PState {}

    async fn observe_count(world: World, Svc(s): Svc<PState>) -> Result<(), AppError> {
        s.last_seen_count.store(world.count(), Ordering::Relaxed);
        Ok(())
    }

    let state = PState::default();
    let (cmd_tx, _cmd_rx) = mpsc::unbounded_channel::<Command>();
    let sender = Sender::new(cmd_tx);
    let app = App::new()
        .handle(Stage::Update, state.clone())
        .handle(Stage::Update, observe_count);

    let cancel = tokio_util::sync::CancellationToken::new();
    let app_state = app.state;
    let world = app.world;
    let pre_handlers = app.pre_handlers;
    let update_handlers = app.update_handlers;

    // First NCN: the intrinsic world folds it in; handler reads count = 1.
    dispatch_cycle(
        Dispatch::Packet(make_ncn(1, "alice")),
        &sender,
        &world,
        None,
        &app_state,
        &pre_handlers,
        &update_handlers,
        &cancel,
    )
    .await;
    assert_eq!(state.last_seen_count.load(Ordering::Relaxed), 1);

    // Second NCN: count rises to 2.
    dispatch_cycle(
        Dispatch::Packet(make_ncn(2, "bob")),
        &sender,
        &world,
        None,
        &app_state,
        &pre_handlers,
        &update_handlers,
        &cancel,
    )
    .await;
    assert_eq!(state.last_seen_count.load(Ordering::Relaxed), 2);

    // External read via the intrinsic world handle reflects the live state.
    assert_eq!(world.count(), 2);
    assert!(world.connection(ConnectionId(1)).is_some());
    assert!(world.connection(ConnectionId(2)).is_some());
}

#[tokio::test]
async fn cancellation_token_extractor_triggers_shutdown() {
    async fn quit_on_ncn(
        _: Packet<Ncn>,
        cancel: tokio_util::sync::CancellationToken,
    ) -> Result<(), AppError> {
        cancel.cancel();
        Ok(())
    }

    let app = App::new().handle(Stage::Update, quit_on_ncn);
    let (cmd_tx, _cmd_rx) = mpsc::unbounded_channel::<Command>();
    let sender = Sender::new(cmd_tx);
    let cancel = tokio_util::sync::CancellationToken::new();
    let app_state = app.state;
    let world = app.world;
    let pre_handlers = app.pre_handlers;
    let update_handlers = app.update_handlers;

    assert!(!cancel.is_cancelled());
    dispatch_cycle(
        Dispatch::Packet(make_ncn(1, "alice")),
        &sender,
        &world,
        None,
        &app_state,
        &pre_handlers,
        &update_handlers,
        &cancel,
    )
    .await;
    assert!(
        cancel.is_cancelled(),
        "handler should have cancelled the runtime token"
    );
}

#[tokio::test]
async fn sender_event_pushes_into_back_channel() {
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

#[tokio::test]
async fn run_if_skips_handler_when_predicate_false() {
    let state = TestState::default();
    let app = App::new()
        .handle(Stage::Update, state.clone())
        .handle(Stage::Update, count_ncn.run_if(|_: State<()>| false));

    let (cmd_tx, _cmd_rx) = mpsc::unbounded_channel::<Command>();
    let sender = Sender::new(cmd_tx);
    let cancel = tokio_util::sync::CancellationToken::new();
    let app_state = app.state;
    let world = app.world;
    let pre_handlers = app.pre_handlers;
    let update_handlers = app.update_handlers;

    dispatch_cycle(
        Dispatch::Packet(make_ncn(1, "alice")),
        &sender,
        &world,
        None,
        &app_state,
        &pre_handlers,
        &update_handlers,
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
    let app = App::new()
        .handle(Stage::Update, state.clone())
        .handle(Stage::Update, count_ncn.run_if(|_: State<()>| true));

    let (cmd_tx, _cmd_rx) = mpsc::unbounded_channel::<Command>();
    let sender = Sender::new(cmd_tx);
    let cancel = tokio_util::sync::CancellationToken::new();
    let app_state = app.state;
    let world = app.world;
    let pre_handlers = app.pre_handlers;
    let update_handlers = app.update_handlers;

    dispatch_cycle(
        Dispatch::Packet(make_ncn(1, "alice")),
        &sender,
        &world,
        None,
        &app_state,
        &pre_handlers,
        &update_handlers,
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
    // A resource that holds a boolean. Handler gated on `in_state`. Flipping
    // the boolean between dispatches changes whether the handler fires.
    use crate::{ExtractCx, FromContext};

    #[derive(Clone)]
    struct Flag(Arc<RwLock<bool>>);

    impl<S> FromContext<S> for Flag {
        fn from_context(cx: &ExtractCx<'_, S>) -> Option<Self> {
            cx.lookup::<Flag>()
        }
    }

    impl<S: Send + Sync + 'static> Handler<(), S> for Flag {}

    let state = TestState::default();
    let flag = Flag(Arc::new(RwLock::new(false)));
    let app = App::new()
        .handle(Stage::Update, state.clone())
        .handle(Stage::Update, flag.clone())
        .handle(
            Stage::Update,
            count_ncn.run_if(|f: Flag, _: State<()>| *f.0.read()),
        );

    let (cmd_tx, _cmd_rx) = mpsc::unbounded_channel::<Command>();
    let sender = Sender::new(cmd_tx);
    let cancel = tokio_util::sync::CancellationToken::new();
    let app_state = app.state;
    let world = app.world;
    let pre_handlers = app.pre_handlers;
    let update_handlers = app.update_handlers;

    // flag = false: handler skipped.
    dispatch_cycle(
        Dispatch::Packet(make_ncn(1, "alice")),
        &sender,
        &world,
        None,
        &app_state,
        &pre_handlers,
        &update_handlers,
        &cancel,
    )
    .await;
    assert_eq!(
        state.ncn_hits.load(Ordering::Relaxed),
        0,
        "should be gated off while flag=false"
    );

    // Flip the flag - the same handler now passes the predicate.
    *flag.0.write() = true;

    dispatch_cycle(
        Dispatch::Packet(make_ncn(2, "bob")),
        &sender,
        &world,
        None,
        &app_state,
        &pre_handlers,
        &update_handlers,
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
async fn periodic_emits_events_on_schedule() {
    use std::time::Duration;

    #[derive(Clone, Debug)]
    struct Tick;

    let mut app = App::new().periodic(Duration::from_millis(5), Tick);

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

    app.cancel.cancel();
}

#[tokio::test]
async fn with_state_holds_typed_value() {
    #[derive(Clone)]
    struct MyState {
        seen: Arc<AtomicUsize>,
        value: i32,
    }

    let seen = Arc::new(AtomicUsize::new(0));
    let state_value = MyState {
        seen: seen.clone(),
        value: 42,
    };

    async fn observe(s: State<MyState>) -> Result<(), AppError> {
        let _ = s.seen.fetch_add(s.value as usize, Ordering::Relaxed);
        Ok(())
    }

    let app = App::with_state(state_value).handle(Stage::Update, observe);

    let (cmd_tx, _cmd_rx) = mpsc::unbounded_channel::<Command>();
    let sender = Sender::new(cmd_tx);
    let cancel = tokio_util::sync::CancellationToken::new();
    let app_state = app.state;
    let world = app.world;
    let pre_handlers = app.pre_handlers;
    let update_handlers = app.update_handlers;

    // A Tiny yields no world events, so the ungated observe handler runs exactly
    // once (an Ncn would also inject a derived Connected dispatch).
    dispatch_cycle(
        Dispatch::Packet(insim::Packet::Tiny(insim::insim::Tiny::default())),
        &sender,
        &world,
        None,
        &app_state,
        &pre_handlers,
        &update_handlers,
        &cancel,
    )
    .await;

    assert_eq!(seen.load(Ordering::Relaxed), 42);
}

#[tokio::test]
async fn state_mutation_visible_across_dispatches() {
    // User picks their own interior-mutability story - here, an atomic.
    #[derive(Clone, Default)]
    struct Counter {
        n: Arc<AtomicUsize>,
    }

    async fn bump(_: Packet<Ncn>, s: State<Counter>) -> Result<(), AppError> {
        let _ = s.n.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }

    let counter = Counter::default();
    let observed = counter.n.clone();
    let app = App::with_state(counter).handle(Stage::Update, bump);

    let (cmd_tx, _cmd_rx) = mpsc::unbounded_channel::<Command>();
    let sender = Sender::new(cmd_tx);
    let cancel = tokio_util::sync::CancellationToken::new();
    let app_state = app.state;
    let world = app.world;
    let pre_handlers = app.pre_handlers;
    let update_handlers = app.update_handlers;

    for i in 1..=3 {
        dispatch_cycle(
            Dispatch::Packet(make_ncn(i, "alice")),
            &sender,
            &world,
            None,
            &app_state,
            &pre_handlers,
            &update_handlers,
            &cancel,
        )
        .await;
    }

    assert_eq!(observed.load(Ordering::Relaxed), 3);
}

#[tokio::test]
async fn state_in_run_if_predicate_gates_handler() {
    // User-owned wrapping - `Flag` holds an `Arc<RwLock<bool>>` itself.
    #[derive(Clone, Default)]
    struct Flag(Arc<RwLock<bool>>);

    let hit_count = Arc::new(AtomicUsize::new(0));
    let hit = hit_count.clone();
    let handler = move |_: Packet<Ncn>| -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<(), AppError>> + Send>,
    > {
        let hit = hit.clone();
        Box::pin(async move {
            let _ = hit.fetch_add(1, Ordering::Relaxed);
            Ok(())
        })
    };

    let flag = Flag::default();
    let flag_inner = flag.0.clone();
    let app = App::with_state(flag).handle(
        Stage::Update,
        handler.run_if(|s: State<Flag>| *s.0.0.read()),
    );

    let (cmd_tx, _cmd_rx) = mpsc::unbounded_channel::<Command>();
    let sender = Sender::new(cmd_tx);
    let cancel = tokio_util::sync::CancellationToken::new();
    let app_state = app.state;
    let world = app.world;
    let pre_handlers = app.pre_handlers;
    let update_handlers = app.update_handlers;

    // Flag off - handler should be gated.
    dispatch_cycle(
        Dispatch::Packet(make_ncn(1, "alice")),
        &sender,
        &world,
        None,
        &app_state,
        &pre_handlers,
        &update_handlers,
        &cancel,
    )
    .await;
    assert_eq!(hit_count.load(Ordering::Relaxed), 0);

    // Flip the flag on.
    *flag_inner.write() = true;

    dispatch_cycle(
        Dispatch::Packet(make_ncn(2, "bob")),
        &sender,
        &world,
        None,
        &app_state,
        &pre_handlers,
        &update_handlers,
        &cancel,
    )
    .await;
    assert_eq!(hit_count.load(Ordering::Relaxed), 1);
}

#[tokio::test]
async fn pre_handler_runs_before_update_handler() {
    // A Pre handler bumps a counter on Packet<Ncn>; an Update handler reads
    // it during the same dispatch. Because Pre handlers run sequentially
    // first, the Update handler must observe the bumped value.
    #[derive(Clone, Default)]
    struct Counter {
        n: Arc<AtomicUsize>,
    }

    impl<S: Send + Sync + 'static> Handler<(), S> for Counter {
        fn call(
            self,
            _cx: &ExtractCx<'_, S>,
        ) -> impl std::future::Future<Output = Result<(), AppError>> + Send {
            async move {
                let _ = self.n.fetch_add(1, Ordering::Relaxed);
                Ok(())
            }
        }
    }

    #[derive(Clone, Default)]
    struct Observed(Arc<AtomicUsize>);
    impl<S: Send + Sync + 'static> Handler<(), S> for Observed {}

    async fn observe(
        _: Packet<Ncn>,
        Svc(c): Svc<Counter>,
        Svc(o): Svc<Observed>,
    ) -> Result<(), AppError> {
        o.0.store(c.n.load(Ordering::Relaxed), Ordering::Relaxed);
        Ok(())
    }

    let counter = Counter::default();
    let observed = Observed::default();
    let app = App::new()
        .handle(Stage::Pre, counter.clone())
        .handle(Stage::Update, observed.clone())
        .handle(Stage::Update, observe);

    let (cmd_tx, _cmd_rx) = mpsc::unbounded_channel::<Command>();
    let sender = Sender::new(cmd_tx);
    let cancel = tokio_util::sync::CancellationToken::new();
    let app_state = app.state;
    let world = app.world;
    let pre_handlers = app.pre_handlers;
    let update_handlers = app.update_handlers;

    dispatch_cycle(
        Dispatch::Packet(make_ncn(1, "alice")),
        &sender,
        &world,
        None,
        &app_state,
        &pre_handlers,
        &update_handlers,
        &cancel,
    )
    .await;

    assert_eq!(observed.0.load(Ordering::Relaxed), 1);
}

#[tokio::test]
async fn pre_handlers_run_sequentially_in_registration_order() {
    // Two Pre handlers append to a shared Vec on Packet<Ncn>; the order in
    // the Vec must match the handler registration order. Distinct newtypes
    // give each handler its own TypeId slot in the IndexMap.
    #[derive(Clone)]
    struct First(Arc<Mutex<Vec<&'static str>>>);
    #[derive(Clone)]
    struct Second(Arc<Mutex<Vec<&'static str>>>);

    impl<S: Send + Sync + 'static> Handler<(), S> for First {
        fn call(
            self,
            _: &ExtractCx<'_, S>,
        ) -> impl std::future::Future<Output = Result<(), AppError>> + Send {
            async move {
                self.0.lock().push("first");
                Ok(())
            }
        }
    }
    impl<S: Send + Sync + 'static> Handler<(), S> for Second {
        fn call(
            self,
            _: &ExtractCx<'_, S>,
        ) -> impl std::future::Future<Output = Result<(), AppError>> + Send {
            async move {
                self.0.lock().push("second");
                Ok(())
            }
        }
    }

    let log = Arc::new(Mutex::new(Vec::new()));
    let app = App::new()
        .handle(Stage::Pre, First(log.clone()))
        .handle(Stage::Pre, Second(log.clone()));

    let (cmd_tx, _cmd_rx) = mpsc::unbounded_channel::<Command>();
    let sender = Sender::new(cmd_tx);
    let cancel = tokio_util::sync::CancellationToken::new();
    let app_state = app.state;
    let world = app.world;
    let pre_handlers = app.pre_handlers;
    let update_handlers = app.update_handlers;

    // A Tiny yields no world events, so the two Pre handlers run exactly once
    // (an Ncn would also inject a derived Connected dispatch, running them again).
    dispatch_cycle(
        Dispatch::Packet(insim::Packet::Tiny(insim::insim::Tiny::default())),
        &sender,
        &world,
        None,
        &app_state,
        &pre_handlers,
        &update_handlers,
        &cancel,
    )
    .await;

    assert_eq!(*log.lock(), vec!["first", "second"]);
}
