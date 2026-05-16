//! Bomb mini-game as a handler-driven state machine, with track setup
//! delegated to `Game::track_rotation` via a single fire-and-forget spawn.
//!
//! Three phases:
//!
//! ```text
//!     Waiting  ─[ Connected, count >= MIN ]──▶  SettingUp
//!     SettingUp ─[ SetupComplete   ]────────▶  Racing
//!     SettingUp ─[ SetupAborted    ]────────▶  Waiting (retry if count >= MIN)
//!     Racing   ─[ Disconnected, count < MIN ]▶ Waiting
//!     Racing   ─[ RaceEnded (server-side) ]──▶ Waiting (retry if count >= MIN)
//! ```
//!
//! Each transition lives in one handler. On entering `SettingUp`,
//! `start_setup` spawns a tokio task that runs `Game::track_rotation` to load
//! the configured track + layout and wait for the race to start. When that
//! task completes successfully it emits a `SetupComplete` synthetic event;
//! on timeout / cancellation / failure it emits `SetupAborted`. The handler
//! returns immediately - the spawn doesn't block the dispatch loop.
//!
//! What's NOT here:
//! - `WaitForPlayers` scene combinator: replaced by the default `Waiting` phase.
//! - `SetupTrack` scene: replaced by the `SettingUp` phase + spawned setup task.
//! - `until_game_ends`: replaced by `on_race_ended` flipping back to `Waiting`.
//! - `loop_until_quit`: replaced by `start_setup` being called again from
//!   `on_race_ended` / `on_setup_aborted` / `on_disconnected` when conditions
//!   permit. The loop is implicit in the state machine.
//! - A hand-rolled tick loop: replaced by `App::periodic(TICK_PERIOD, BombTick)`.
//!
//! Run with:
//!     cargo run -p insim_app --example bomb -- 127.0.0.1:29999
//!     cargo run -p insim_app --example bomb -- 127.0.0.1:29999 --track BL1
//!     cargo run -p insim_app --example bomb -- 127.0.0.1:29999 --admin-password hunter2

use std::{
    collections::HashMap,
    str::FromStr,
    sync::{Arc, Mutex, RwLock},
    time::{Duration, Instant},
};

use clap::Parser;
use fixedbitset as _;
use futures as _;
use insim::{
    Colour,
    core::{
        object::{
            ObjectInfo,
            insim::{InsimCheckpoint, InsimCheckpointKind},
        },
        track::Track,
    },
    identifiers::{ConnectionId, PlayerId},
    insim::{BtnStyle, Con, Crs, Npl, Pit, PlayerType, Pll, RaceLaps, Tiny, TinyType, Toc, Uco},
};
use insim_app::{
    App, AppError, Connected, Disconnected, Event, ExtractCx, Extension, FromContext, Game,
    HandlerExt, Packet, Presence, RaceEnded, Sender, serve,
    ui::{self, Component, InvalidateHandle, Ui},
    util::mtc,
};
use taffy as _;
use thiserror as _;
use tokio_util::sync::CancellationToken;
use tracing as _;

// ---------------------------------------------------------------------------
// Game tuning
// ---------------------------------------------------------------------------

const MIN_PLAYERS: usize = 2;
const TICK_PERIOD: Duration = Duration::from_millis(500);
const COLLISION_THRESHOLD_MPS: f32 = 30.0;

#[derive(Clone, Debug)]
struct BombConfig {
    checkpoint_timeout: Duration,
    checkpoint_penalty: Duration,
    collision_max_penalty: Duration,
    track: Track,
    layout: Option<String>,
    setup_timeout: Duration,
}

impl Default for BombConfig {
    fn default() -> Self {
        Self {
            checkpoint_timeout: Duration::from_secs(30),
            checkpoint_penalty: Duration::from_millis(250),
            collision_max_penalty: Duration::from_millis(500),
            track: Track::default(),
            layout: None,
            setup_timeout: Duration::from_secs(60),
        }
    }
}

// ---------------------------------------------------------------------------
// Game state (verbatim from the prior version)
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
struct ActiveRun {
    started_at: Instant,
    deadline: Instant,
    current_timeout: Duration,
    checkpoints: i64,
    uname: String,
    pname: String,
    ucid: ConnectionId,
}

impl ActiveRun {
    fn new(
        uname: String,
        pname: String,
        ucid: ConnectionId,
        config: &BombConfig,
        now: Instant,
    ) -> Self {
        Self {
            started_at: now,
            deadline: now + config.checkpoint_timeout,
            current_timeout: config.checkpoint_timeout,
            checkpoints: 0,
            uname,
            pname,
            ucid,
        }
    }

    fn survival_ms(&self, now: Instant) -> i64 {
        (self.deadline.min(now) - self.started_at).as_millis() as i64
    }

    fn time_left(&self, now: Instant) -> Duration {
        self.deadline.saturating_duration_since(now)
    }
}

#[derive(Clone, Debug)]
struct PlayerInfo {
    ucid: ConnectionId,
    pname: String,
    uname: String,
    ptype: PlayerType,
}

#[derive(Debug)]
enum CheckpointOutcome {
    Started {
        ucid: ConnectionId,
    },
    Refreshed {
        ucid: ConnectionId,
        checkpoints: i64,
        new_window: Duration,
    },
    Extended {
        ucid: ConnectionId,
        checkpoints: i64,
        time_left: Duration,
    },
}

struct BombState {
    config: BombConfig,
    active_runs: HashMap<PlayerId, ActiveRun>,
    /// In-memory leaderboard for the current round. Each entry is
    /// `(uname, checkpoints, survival_ms)`. Sorted by checkpoints desc,
    /// survival_ms desc.
    leaderboard: Vec<(String, i64, i64)>,
}

impl BombState {
    fn new(config: BombConfig) -> Self {
        Self {
            config,
            active_runs: HashMap::new(),
            leaderboard: Vec::new(),
        }
    }

    fn on_checkpoint(
        &mut self,
        p: &PlayerInfo,
        plid: PlayerId,
        is_finish: bool,
        now: Instant,
    ) -> Option<CheckpointOutcome> {
        if let Some(run) = self.active_runs.get_mut(&plid) {
            let outcome = if is_finish {
                run.current_timeout = self.config.checkpoint_timeout;
                run.deadline = now + self.config.checkpoint_timeout;
                run.checkpoints += 1;
                CheckpointOutcome::Refreshed {
                    ucid: run.ucid,
                    checkpoints: run.checkpoints,
                    new_window: self.config.checkpoint_timeout,
                }
            } else {
                run.deadline = now + run.current_timeout;
                run.current_timeout = run
                    .current_timeout
                    .saturating_sub(self.config.checkpoint_penalty);
                run.checkpoints += 1;
                CheckpointOutcome::Extended {
                    ucid: run.ucid,
                    checkpoints: run.checkpoints,
                    time_left: run.time_left(now),
                }
            };
            return Some(outcome);
        }
        if p.ptype.contains(PlayerType::AI) {
            return None;
        }
        let _ = self.active_runs.insert(
            plid,
            ActiveRun::new(p.uname.clone(), p.pname.clone(), p.ucid, &self.config, now),
        );
        Some(CheckpointOutcome::Started { ucid: p.ucid })
    }

    fn on_collision(
        &mut self,
        plid: PlayerId,
        speed_diff_mps: f32,
        now: Instant,
    ) -> Option<(ConnectionId, Duration, Duration)> {
        let fraction = (speed_diff_mps / COLLISION_THRESHOLD_MPS).clamp(0.0, 1.0);
        let penalty = Duration::from_millis(
            (fraction * self.config.collision_max_penalty.as_millis() as f32) as u64,
        );
        if penalty.is_zero() {
            return None;
        }
        let run = self.active_runs.get_mut(&plid)?;
        run.deadline = run.deadline.checked_sub(penalty).unwrap_or(now);
        Some((run.ucid, penalty, run.time_left(now)))
    }

    fn on_reset(
        &mut self,
        plid: PlayerId,
        now: Instant,
    ) -> Option<(ConnectionId, Duration, Duration)> {
        let penalty = self.config.checkpoint_penalty;
        let run = self.active_runs.get_mut(&plid)?;
        run.deadline = run.deadline.checked_sub(penalty).unwrap_or(now);
        Some((run.ucid, penalty, run.time_left(now)))
    }

    fn finalize(&mut self, run: &ActiveRun, survival_ms: i64) {
        self.leaderboard
            .push((run.uname.clone(), run.checkpoints, survival_ms));
        self.leaderboard
            .sort_by(|a, b| b.1.cmp(&a.1).then(b.2.cmp(&a.2)));
        self.leaderboard.truncate(10);
    }

    fn tick_expired(&mut self, now: Instant) -> Vec<ActiveRun> {
        let expired: Vec<PlayerId> = self
            .active_runs
            .iter()
            .filter(|(_, r)| r.deadline < now)
            .map(|(k, _)| *k)
            .collect();
        expired
            .into_iter()
            .filter_map(|k| self.active_runs.remove(&k))
            .collect()
    }

    fn snapshot(&self, phase: &str) -> BombGlobal {
        let now = Instant::now();
        let mut active: Vec<_> = self
            .active_runs
            .values()
            .map(|r| (r.pname.clone(), r.checkpoints, r.time_left(now)))
            .collect();
        active.sort_by_key(|r| std::cmp::Reverse(r.1));
        BombGlobal {
            phase: phase.to_string(),
            leaderboard: self.leaderboard.clone(),
            active_runs: active,
        }
    }
}

// ---------------------------------------------------------------------------
// UI (verbatim)
// ---------------------------------------------------------------------------

#[derive(Clone, Default, Debug)]
struct BombGlobal {
    phase: String,
    leaderboard: Vec<(String, i64, i64)>,
    /// (pname, checkpoints, time_left)
    active_runs: Vec<(String, i64, Duration)>,
}

#[derive(Clone, Debug)]
enum BombMsg {}

struct BombView {
    _invalidator: InvalidateHandle,
}

impl Component for BombView {
    type Message = BombMsg;
    type Props<'a> = (&'a BombGlobal, &'a ());

    fn render(&self, (global, _): Self::Props<'_>) -> ui::Node<Self::Message> {
        let active_rows: Vec<_> = global
            .active_runs
            .iter()
            .map(|(p, cps, time_left)| {
                ui::text(
                    format!("{p}: {cps} cps  {:.1}s", time_left.as_secs_f64()),
                    BtnStyle::default(),
                )
                .w(50.0)
                .h(5.0)
            })
            .collect();

        let leaderboard_rows: Vec<_> = global
            .leaderboard
            .iter()
            .take(5)
            .enumerate()
            .map(|(i, (u, cps, ms))| {
                ui::text(
                    format!("{}. {u}: {cps} cps ({:.1}s)", i + 1, *ms as f64 / 1000.0),
                    BtnStyle::default(),
                )
                .w(50.0)
                .h(5.0)
            })
            .collect();

        ui::container()
            .flex()
            .flex_col()
            .with_child(
                ui::text(format!("Bomb - {}", global.phase), BtnStyle::default())
                    .w(50.0)
                    .h(5.0),
            )
            .with_child(ui::text("Active runs", BtnStyle::default()).w(50.0).h(5.0))
            .with_children(active_rows)
            .with_child(ui::text("Top runs", BtnStyle::default()).w(50.0).h(5.0))
            .with_children(leaderboard_rows)
    }
}

type BombUi = Ui<BombView, BombGlobal, ()>;

// ---------------------------------------------------------------------------
// Phase extension - the state-machine cursor + in-flight setup cancel handle.
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
enum BombPhase {
    #[default]
    Waiting,
    SettingUp,
    Racing,
}

impl BombPhase {
    fn label(self) -> &'static str {
        match self {
            BombPhase::Waiting => "waiting for players",
            BombPhase::SettingUp => "setting up track",
            BombPhase::Racing => "racing",
        }
    }
}

struct PhaseInner {
    state: BombPhase,
    /// Set while `state == SettingUp` so any handler can cancel the in-flight
    /// `track_rotation` spawn (e.g. when players drop below threshold).
    setup_cancel: Option<CancellationToken>,
    /// Runtime-wide cancel token; child tokens for setup tasks derive from this
    /// so they die when the runtime dies.
    runtime_cancel: CancellationToken,
}

#[derive(Clone)]
struct Phase(Arc<RwLock<PhaseInner>>);

impl Phase {
    fn new(runtime_cancel: CancellationToken) -> Self {
        Self(Arc::new(RwLock::new(PhaseInner {
            state: BombPhase::default(),
            setup_cancel: None,
            runtime_cancel,
        })))
    }

    fn get(&self) -> BombPhase {
        self.0.read().expect("poison").state
    }

    fn set(&self, p: BombPhase) {
        self.0.write().expect("poison").state = p;
    }

    fn is_racing(&self) -> bool {
        self.get() == BombPhase::Racing
    }

    /// Install a fresh child cancel token for an in-flight setup task. The
    /// returned token shares cancellation with the one stored on `self`, so
    /// `cancel_setup` can interrupt the task from another handler.
    fn make_setup_cancel(&self) -> CancellationToken {
        let mut guard = self.0.write().expect("poison");
        let token = guard.runtime_cancel.child_token();
        guard.setup_cancel = Some(token.clone());
        token
    }

    /// Cancel and clear the stored setup token. No-op if none is set.
    fn cancel_setup(&self) {
        let mut guard = self.0.write().expect("poison");
        if let Some(c) = guard.setup_cancel.take() {
            c.cancel();
        }
    }

    /// Forget the stored setup token without cancelling it. Used by
    /// `on_setup_complete` when the setup task succeeded.
    fn clear_setup_cancel(&self) {
        self.0.write().expect("poison").setup_cancel = None;
    }
}

impl<S: Send + Sync + 'static> Extension<S> for Phase {}

impl<S: Send + Sync + 'static> FromContext<S> for Phase {
    fn from_context(cx: &ExtractCx<'_, S>) -> Option<Self> {
        cx.extensions.get::<Phase>()
    }
}

// ---------------------------------------------------------------------------
// Bomb extension - the round's mutable game data.
// ---------------------------------------------------------------------------

struct BombInner {
    state: BombState,
    players: HashMap<PlayerId, PlayerInfo>,
}

#[derive(Clone)]
struct Bomb {
    inner: Arc<Mutex<BombInner>>,
}

impl Bomb {
    fn new(config: BombConfig) -> Self {
        Self {
            inner: Arc::new(Mutex::new(BombInner {
                state: BombState::new(config),
                players: HashMap::new(),
            })),
        }
    }

    fn config_snapshot(&self) -> BombConfig {
        self.inner.lock().expect("poison").state.config.clone()
    }
}

impl<S: Send + Sync + 'static> Extension<S> for Bomb {}

impl<S: Send + Sync + 'static> FromContext<S> for Bomb {
    fn from_context(cx: &ExtractCx<'_, S>) -> Option<Self> {
        cx.extensions.get::<Bomb>()
    }
}

fn refresh_ui(bomb: &Bomb, phase: BombPhase, ui: &BombUi) {
    let snapshot = bomb
        .inner
        .lock()
        .expect("poison")
        .state
        .snapshot(phase.label());
    ui.assign(snapshot);
}

// ---------------------------------------------------------------------------
// Synthetic events local to bomb
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
struct BombTick;

/// Emitted by the spawned setup task when `Game::track_rotation` completes
/// successfully and the race is in progress.
#[derive(Clone, Debug)]
struct SetupComplete;

/// Emitted by the spawned setup task when `track_rotation` was cancelled,
/// timed out, or returned `None` for any other reason.
#[derive(Clone, Debug)]
struct SetupAborted;

// ---------------------------------------------------------------------------
// Setup: spawn `Game::track_rotation` and signal completion via synthetic events.
// ---------------------------------------------------------------------------

fn start_setup(phase: &Phase, game: &Game, sender: &Sender, bomb: &Bomb, ui: &BombUi) {
    let cfg = bomb.config_snapshot();
    let setup_cancel = phase.make_setup_cancel();
    phase.set(BombPhase::SettingUp);
    let _ = sender.packet(mtc(
        "Bomb - setting up track, hit /ready when prompted.",
        Some(ConnectionId::ALL),
    ));
    refresh_ui(bomb, BombPhase::SettingUp, ui);

    let game = game.clone();
    let sender = sender.clone();
    // Fire-and-forget: the spawn's only side effect is a synthetic event when
    // it completes; the JoinHandle is intentionally dropped.
    drop(tokio::spawn(async move {
        let result = tokio::select! {
            r = game.track_rotation(cfg.track, RaceLaps::Untimed, 0, cfg.layout, setup_cancel.clone()) => r,
            _ = tokio::time::sleep(cfg.setup_timeout) => None,
        };
        match result {
            Some(()) => { let _ = sender.event(SetupComplete); },
            None => { let _ = sender.event(SetupAborted); },
        }
    }));
}

// ---------------------------------------------------------------------------
// Phase-transition handlers
// ---------------------------------------------------------------------------

async fn on_connected(
    _: Event<Connected>,
    presence: Presence,
    phase: Phase,
    bomb: Bomb,
    ui: BombUi,
    game: Game,
    sender: Sender,
) -> Result<(), AppError> {
    if phase.get() != BombPhase::Waiting || presence.count() < MIN_PLAYERS {
        refresh_ui(&bomb, phase.get(), &ui);
        return Ok(());
    }
    start_setup(&phase, &game, &sender, &bomb, &ui);
    Ok(())
}

async fn on_disconnected(
    _: Event<Disconnected>,
    presence: Presence,
    phase: Phase,
    bomb: Bomb,
    sender: Sender,
    ui: BombUi,
) -> Result<(), AppError> {
    if presence.count() >= MIN_PLAYERS {
        refresh_ui(&bomb, phase.get(), &ui);
        return Ok(());
    }
    match phase.get() {
        BombPhase::Waiting => {
            refresh_ui(&bomb, BombPhase::Waiting, &ui);
        },
        BombPhase::SettingUp => {
            // Cancel the in-flight setup; the spawn will emit SetupAborted
            // which `on_setup_aborted` translates into Waiting + retry.
            phase.cancel_setup();
            refresh_ui(&bomb, BombPhase::SettingUp, &ui);
        },
        BombPhase::Racing => {
            // End the round: finalize every active run and reset.
            let now = Instant::now();
            let runs: Vec<ActiveRun> = {
                let mut guard = bomb.inner.lock().expect("poison");
                guard.players.clear();
                guard.state.active_runs.drain().map(|(_, r)| r).collect()
            };
            for run in &runs {
                let _ = sender.packet(mtc(
                    format!("Run ended - left race after {} cps", run.checkpoints).red(),
                    Some(run.ucid),
                ));
            }
            {
                let mut guard = bomb.inner.lock().expect("poison");
                for run in &runs {
                    let survival_ms = run.survival_ms(now);
                    guard.state.finalize(run, survival_ms);
                }
            }
            phase.set(BombPhase::Waiting);
            sender.packet(mtc(
                "Bomb - not enough players, restarting.",
                Some(ConnectionId::ALL),
            ))?;
            refresh_ui(&bomb, BombPhase::Waiting, &ui);
        },
    }
    Ok(())
}

async fn on_setup_complete(
    _: Event<SetupComplete>,
    phase: Phase,
    bomb: Bomb,
    sender: Sender,
    ui: BombUi,
) -> Result<(), AppError> {
    if phase.get() != BombPhase::SettingUp {
        return Ok(());
    }
    phase.set(BombPhase::Racing);
    phase.clear_setup_cancel();

    let cfg_secs = bomb.config_snapshot().checkpoint_timeout.as_secs_f64();
    sender.packet(mtc(
        format!("Bomb - hit checkpoints before the {cfg_secs:.0}s timer expires!"),
        Some(ConnectionId::ALL),
    ))?;
    // Ask LFS to resend Npl for every player currently in the race so the
    // bomb players map repopulates without waiting for the next join.
    sender.packet(insim::Packet::Tiny(Tiny {
        subt: TinyType::Npl,
        ..Default::default()
    }))?;
    refresh_ui(&bomb, BombPhase::Racing, &ui);
    Ok(())
}

async fn on_setup_aborted(
    _: Event<SetupAborted>,
    phase: Phase,
    presence: Presence,
    bomb: Bomb,
    ui: BombUi,
    game: Game,
    sender: Sender,
) -> Result<(), AppError> {
    if phase.get() != BombPhase::SettingUp {
        return Ok(());
    }
    phase.set(BombPhase::Waiting);
    phase.clear_setup_cancel();
    sender.packet(mtc(
        "Bomb - setup failed, restarting.",
        Some(ConnectionId::ALL),
    ))?;
    refresh_ui(&bomb, BombPhase::Waiting, &ui);
    // Auto-retry if we still have the players.
    if presence.count() >= MIN_PLAYERS {
        start_setup(&phase, &game, &sender, &bomb, &ui);
    }
    Ok(())
}

async fn on_race_ended(
    _: Event<RaceEnded>,
    phase: Phase,
    presence: Presence,
    bomb: Bomb,
    ui: BombUi,
    game: Game,
    sender: Sender,
) -> Result<(), AppError> {
    if phase.get() != BombPhase::Racing {
        return Ok(());
    }
    // Finalize any still-active runs as "race ended" instead of timed out.
    let now = Instant::now();
    let runs: Vec<ActiveRun> = {
        let mut guard = bomb.inner.lock().expect("poison");
        guard.players.clear();
        guard.state.active_runs.drain().map(|(_, r)| r).collect()
    };
    for run in &runs {
        let _ = sender.packet(mtc(
            format!("Run ended - race finished after {} cps", run.checkpoints).yellow(),
            Some(run.ucid),
        ));
    }
    {
        let mut guard = bomb.inner.lock().expect("poison");
        for run in &runs {
            let survival_ms = run.survival_ms(now);
            guard.state.finalize(run, survival_ms);
        }
    }
    phase.set(BombPhase::Waiting);
    refresh_ui(&bomb, BombPhase::Waiting, &ui);
    if presence.count() >= MIN_PLAYERS {
        start_setup(&phase, &game, &sender, &bomb, &ui);
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// In-round handlers (run_if(in_state(while_racing)))
// ---------------------------------------------------------------------------

async fn on_npl(Packet(npl): Packet<Npl>, presence: Presence, bomb: Bomb) -> Result<(), AppError> {
    let uname = presence
        .get(npl.ucid)
        .map(|c| c.uname)
        .unwrap_or_default();
    let _ = bomb.inner.lock().expect("poison").players.insert(
        npl.plid,
        PlayerInfo {
            ucid: npl.ucid,
            pname: npl.pname.clone(),
            uname,
            ptype: npl.ptype,
        },
    );
    Ok(())
}

async fn on_pll(
    Packet(pll): Packet<Pll>,
    phase: Phase,
    bomb: Bomb,
    sender: Sender,
    ui: BombUi,
) -> Result<(), AppError> {
    let now = Instant::now();
    let run = {
        let mut guard = bomb.inner.lock().expect("poison");
        let _ = guard.players.remove(&pll.plid);
        guard.state.active_runs.remove(&pll.plid)
    };
    if let Some(run) = run {
        let survival_ms = run.survival_ms(now);
        sender.packet(mtc(
            format!("Run ended - left race after {} cps", run.checkpoints).red(),
            Some(run.ucid),
        ))?;
        bomb.inner
            .lock()
            .expect("poison")
            .state
            .finalize(&run, survival_ms);
        refresh_ui(&bomb, phase.get(), &ui);
    }
    Ok(())
}

async fn on_toc(Packet(toc): Packet<Toc>, bomb: Bomb) -> Result<(), AppError> {
    let mut guard = bomb.inner.lock().expect("poison");
    if let Some(p) = guard.players.get_mut(&toc.plid) {
        p.ucid = toc.newucid;
    }
    if let Some(r) = guard.state.active_runs.get_mut(&toc.plid) {
        r.ucid = toc.newucid;
    }
    Ok(())
}

async fn on_pit(
    Packet(pit): Packet<Pit>,
    phase: Phase,
    bomb: Bomb,
    sender: Sender,
    ui: BombUi,
) -> Result<(), AppError> {
    let now = Instant::now();
    let run = {
        let mut guard = bomb.inner.lock().expect("poison");
        guard.state.active_runs.remove(&pit.plid)
    };
    if let Some(run) = run {
        let survival_ms = run.survival_ms(now);
        sender.packet(mtc(
            format!(
                "PITTED - run ended after {} cps. Commit to your fuel.",
                run.checkpoints
            )
            .red(),
            Some(run.ucid),
        ))?;
        bomb.inner
            .lock()
            .expect("poison")
            .state
            .finalize(&run, survival_ms);
        refresh_ui(&bomb, phase.get(), &ui);
    }
    Ok(())
}

async fn on_crs(
    Packet(crs): Packet<Crs>,
    phase: Phase,
    bomb: Bomb,
    sender: Sender,
    ui: BombUi,
) -> Result<(), AppError> {
    let now = Instant::now();
    let result = bomb.inner.lock().expect("poison").state.on_reset(crs.plid, now);
    if let Some((ucid, penalty, time_left)) = result {
        sender.packet(mtc(
            format!(
                "PENALTY -{:.2}s - {:.1}s left",
                penalty.as_secs_f64(),
                time_left.as_secs_f64()
            )
            .red(),
            Some(ucid),
        ))?;
        refresh_ui(&bomb, phase.get(), &ui);
    }
    Ok(())
}

async fn on_con(
    Packet(con): Packet<Con>,
    phase: Phase,
    bomb: Bomb,
    sender: Sender,
    ui: BombUi,
) -> Result<(), AppError> {
    let now = Instant::now();
    let mps = con.spclose.to_meters_per_sec();
    let mut any_hit = false;
    for plid in [con.a.plid, con.b.plid] {
        let result = bomb
            .inner
            .lock()
            .expect("poison")
            .state
            .on_collision(plid, mps, now);
        if let Some((ucid, penalty, time_left)) = result {
            sender.packet(mtc(
                format!(
                    "PENALTY -{:.2}s - {:.1}s left",
                    penalty.as_secs_f64(),
                    time_left.as_secs_f64()
                )
                .red(),
                Some(ucid),
            ))?;
            any_hit = true;
        }
    }
    if any_hit {
        refresh_ui(&bomb, phase.get(), &ui);
    }
    Ok(())
}

async fn on_uco(
    Packet(uco): Packet<Uco>,
    phase: Phase,
    bomb: Bomb,
    sender: Sender,
    ui: BombUi,
) -> Result<(), AppError> {
    let ObjectInfo::InsimCheckpoint(InsimCheckpoint { kind, .. }) = uco.info else {
        return Ok(());
    };
    let is_finish = matches!(kind, InsimCheckpointKind::Finish);
    let now = Instant::now();

    let outcome = {
        let mut guard = bomb.inner.lock().expect("poison");
        let player = match guard.players.get(&uco.plid).cloned() {
            Some(p) => p,
            None => return Ok(()),
        };
        guard.state.on_checkpoint(&player, uco.plid, is_finish, now)
    };
    let Some(outcome) = outcome else {
        return Ok(());
    };

    match outcome {
        CheckpointOutcome::Started { ucid } => {
            sender.packet(mtc(
                "Run started - hit every checkpoint!".light_green(),
                Some(ucid),
            ))?;
        },
        CheckpointOutcome::Refreshed {
            ucid,
            checkpoints,
            new_window,
        } => {
            sender.packet(mtc(
                format!(
                    "FINISH - cp {checkpoints} - REFRESHED {:.1}s",
                    new_window.as_secs_f64()
                )
                .yellow(),
                Some(ucid),
            ))?;
        },
        CheckpointOutcome::Extended {
            ucid,
            checkpoints,
            time_left,
        } => {
            sender.packet(mtc(
                format!("cp {checkpoints} - {:.1}s left", time_left.as_secs_f64())
                    .light_green(),
                Some(ucid),
            ))?;
        },
    }
    refresh_ui(&bomb, phase.get(), &ui);
    Ok(())
}

async fn on_tick(
    _: Event<BombTick>,
    phase: Phase,
    bomb: Bomb,
    sender: Sender,
    ui: BombUi,
) -> Result<(), AppError> {
    let now = Instant::now();
    let expired = bomb.inner.lock().expect("poison").state.tick_expired(now);
    let had_expired = !expired.is_empty();
    let mut has_active = false;
    for run in &expired {
        let survival_ms = run.survival_ms(now);
        let _ = sender.packet(mtc(
            format!(
                "BOOM - {} cps, {:.1}s",
                run.checkpoints,
                survival_ms as f64 / 1000.0
            )
            .red(),
            Some(run.ucid),
        ));
        bomb.inner
            .lock()
            .expect("poison")
            .state
            .finalize(run, survival_ms);
    }
    if !had_expired {
        has_active = !bomb
            .inner
            .lock()
            .expect("poison")
            .state
            .active_runs
            .is_empty();
    }
    if had_expired || has_active {
        refresh_ui(&bomb, phase.get(), &ui);
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Wiring
// ---------------------------------------------------------------------------

#[derive(Parser, Debug)]
#[command(about = "insim_app - bomb game example (handler-driven)")]
struct Args {
    /// LFS InSim address (host:port).
    #[arg(long, default_value = "127.0.0.1:29999")]
    addr: String,

    /// InSim admin password, if the host requires one.
    #[arg(long)]
    admin_password: Option<String>,

    /// LFS track code (e.g. BL1, AS1, SO1).
    #[arg(long, default_value = "FE1X")]
    track: String,

    /// Optional autocross layout name (loaded via /axload).
    #[arg(long)]
    layout: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), AppError> {
    tracing_subscriber::fmt::init();
    let args = Args::parse();

    let track = Track::from_str(&args.track).expect("unknown track code");
    let config = BombConfig {
        track,
        layout: args.layout.clone(),
        ..BombConfig::default()
    };

    let app = App::<()>::new();
    let sender = app.sender().clone();
    let runtime_cancel = app.cancel_token().clone();

    let ui = Ui::<BombView, BombGlobal, ()>::new(
        sender.clone(),
        BombGlobal {
            phase: BombPhase::Waiting.label().to_string(),
            ..Default::default()
        },
        |_ucid, invalidator| BombView {
            _invalidator: invalidator,
        },
    );

    let presence = Presence::new(sender.clone());
    let game = Game::new(sender.clone());
    let phase = Phase::new(runtime_cancel);
    let bomb = Bomb::new(config);

    // Predicate that gates in-round handlers. Annotating the closure's `cx`
    // parameter pins `S = ()` so the framework's generic `in_state` can infer
    // the rest from the closure's typed argument.
    let while_racing = |cx: &ExtractCx<'_, ()>| {
        Phase::from_context(cx).is_some_and(|p| p.is_racing())
    };

    let app = app
        .with_state(())
        .extension(presence.clone())
        .extension(game.clone())
        .extension(ui.clone())
        .extension(phase.clone())
        .extension(bomb.clone())
        // Phase transitions.
        .handler(on_connected)
        .handler(on_disconnected)
        .handler(on_setup_complete)
        .handler(on_setup_aborted)
        .handler(on_race_ended)
        // In-round handlers - only run while we're racing.
        .handler(on_npl.run_if(while_racing))
        .handler(on_pll.run_if(while_racing))
        .handler(on_toc.run_if(while_racing))
        .handler(on_pit.run_if(while_racing))
        .handler(on_crs.run_if(while_racing))
        .handler(on_con.run_if(while_racing))
        .handler(on_uco.run_if(while_racing))
        .handler(on_tick.run_if(while_racing))
        // Background ticker emitting BombTick every TICK_PERIOD.
        .periodic(TICK_PERIOD, BombTick);

    let builder = insim::tcp(args.addr)
        .isi_iname("bomb".to_string())
        .isi_prefix('!')
        .isi_admin_password(args.admin_password);

    serve(builder, app).await
}
