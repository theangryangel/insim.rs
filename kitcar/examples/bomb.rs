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
//! The whole game state - phase machine, active runs, players, leaderboard -
//! lives in one `Bomb` value installed via `App::with_state(bomb)`. Handlers
//! extract `State<Bomb>` and lock briefly to mutate. There is no Resources
//! entry for the bomb itself, and no separate `Phase` value: the phase is
//! just a field on `Bomb`.
//!
//! Run with:
//!     cargo run -p kitcar --example bomb -- 127.0.0.1:29999
//!     cargo run -p kitcar --example bomb -- 127.0.0.1:29999 --track BL1
//!     cargo run -p kitcar --example bomb -- 127.0.0.1:29999 --admin-password hunter2

use std::{
    collections::HashMap,
    str::FromStr,
    sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
    time::{Duration, Instant},
};

use clap::Parser;
use futures as _;
use indexmap as _;
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
    insim::{BtnStyle, Con, Crs, Npl, Pit, PlayerType, RaceLaps, Tiny, TinyType, Toc, Uco},
};
use insim_extra as _;
use kitcar::{
    App, AppError, Connected, Disconnected, Event, Game, HandlerExt, Packet, PenaltyClearer,
    PlayerLeft, PlayerTeleportedToPits, Presence, RaceEnded, Sender, Stage, State, run,
    track_rotation,
    ui::{self, Component, Ui},
    util::mtc,
};
use thiserror as _;
use tokio_util::sync::CancellationToken;
use tracing as _;

const MIN_PLAYERS: usize = 2;
const TICK_PERIOD: Duration = Duration::from_millis(500);
const COLLISION_THRESHOLD_MPS: f32 = 30.0;
const PENALTY_CLEAR_DELAY: Duration = Duration::from_secs(15);

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
        uname: String,
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

struct BombInner {
    config: BombConfig,
    phase: BombPhase,
    /// Set while `phase == SettingUp` so any handler can cancel the in-flight
    /// `track_rotation` spawn (e.g. when players drop below threshold).
    setup_cancel: Option<CancellationToken>,
    /// Runtime-wide cancel token; child tokens for setup tasks derive from this
    /// so they die when the runtime dies.
    runtime_cancel: CancellationToken,
    active_runs: HashMap<PlayerId, ActiveRun>,
    players: HashMap<PlayerId, PlayerInfo>,
    /// In-memory leaderboard for the current round. Each entry is
    /// `(uname, pname, checkpoints, survival_ms)`. Sorted by checkpoints
    /// desc, survival_ms desc.
    leaderboard: Vec<(String, String, i64, i64)>,
}

/// Cheap-clone handle over the shared game state. The framework no longer
/// wraps `App::with_state(…)` in a lock; this is the user-owned wrapping.
#[derive(Clone)]
struct Bomb {
    inner: Arc<RwLock<BombInner>>,
}

impl Bomb {
    fn new(config: BombConfig, runtime_cancel: CancellationToken) -> Self {
        Self {
            inner: Arc::new(RwLock::new(BombInner {
                config,
                phase: BombPhase::default(),
                setup_cancel: None,
                runtime_cancel,
                active_runs: HashMap::new(),
                players: HashMap::new(),
                leaderboard: Vec::new(),
            })),
        }
    }

    fn read(&self) -> RwLockReadGuard<'_, BombInner> {
        self.inner.read().expect("poison")
    }

    fn write(&self) -> RwLockWriteGuard<'_, BombInner> {
        self.inner.write().expect("poison")
    }
}

impl BombInner {
    /// Install a fresh child cancel token for an in-flight setup task. The
    /// returned token shares cancellation with the one stored on `self`, so
    /// another handler can interrupt the task via [`Bomb::cancel_setup`].
    fn make_setup_cancel(&mut self) -> CancellationToken {
        let token = self.runtime_cancel.child_token();
        self.setup_cancel = Some(token.clone());
        token
    }

    /// Cancel and clear the stored setup token. No-op if none is set.
    fn cancel_setup(&mut self) {
        if let Some(c) = self.setup_cancel.take() {
            c.cancel();
        }
    }

    /// Forget the stored setup token without cancelling it. Used on successful
    /// setup completion.
    fn clear_setup_cancel(&mut self) {
        self.setup_cancel = None;
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
        Some(CheckpointOutcome::Started {
            ucid: p.ucid,
            uname: p.uname.clone(),
        })
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
        self.leaderboard.push((
            run.uname.clone(),
            run.pname.clone(),
            run.checkpoints,
            survival_ms,
        ));
        self.leaderboard
            .sort_by(|a, b| b.2.cmp(&a.2).then(b.3.cmp(&a.3)));
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

    fn snapshot(&self) -> BombGlobal {
        let mut active: Vec<_> = self
            .active_runs
            .values()
            .map(|r| {
                (
                    r.uname.clone(),
                    r.pname.clone(),
                    r.checkpoints,
                    r.deadline,
                    r.current_timeout,
                )
            })
            .collect();
        active.sort_by_key(|r| std::cmp::Reverse(r.2));
        BombGlobal {
            phase: self.phase.label().to_string(),
            leaderboard: self.leaderboard.clone(),
            active_runs: active,
        }
    }
}

fn hud_title() -> BtnStyle {
    BtnStyle::default().dark().yellow()
}

fn hud_text() -> BtnStyle {
    BtnStyle::default().dark().light_grey()
}

fn hud_active() -> BtnStyle {
    BtnStyle::default().dark().white()
}

fn hud_muted() -> BtnStyle {
    BtnStyle::default().dark().grey()
}

fn topbar<Msg>(title: &str) -> ui::Node<Msg> {
    ui::container()
        .flex()
        .flex_row()
        .justify_center()
        .with_child(ui::text(title, hud_title()).w(30.0).h(5.0))
}

#[derive(Clone, Default, Debug)]
struct BombGlobal {
    phase: String,
    /// (uname, pname, cps, survival_ms) - sorted by cps desc, survival desc.
    leaderboard: Vec<(String, String, i64, i64)>,
    /// (uname, pname, cps, deadline, current_timeout) - live values; the view
    /// computes the remaining fraction on each render.
    active_runs: Vec<(String, String, i64, Instant, Duration)>,
}

#[derive(Clone, Default, Debug)]
struct BombConnectionProps {
    uname: String,
    in_run: bool,
}

#[derive(Clone, Debug)]
enum BombMsg {}

struct BombView {
    // Held only to keep the redraw ticker alive for this view's lifetime.
    _tick_handle: tokio::task::JoinHandle<()>,
}

impl Component for BombView {
    type Message = BombMsg;
    type Props<'a> = (&'a BombGlobal, &'a BombConnectionProps);

    fn render(&self, (global, player): Self::Props<'_>) -> ui::Node<Self::Message> {
        let status_str = if player.in_run { "In run" } else { "Waiting" };
        let status_style = if player.in_run {
            hud_active()
        } else {
            hud_muted()
        };

        let now = Instant::now();
        let active_run_rows: Vec<ui::Node<BombMsg>> = global
            .active_runs
            .iter()
            .map(|(uname, pname, cps, deadline, current_timeout)| {
                let secs_left = deadline.saturating_duration_since(now).as_secs_f64();
                let fraction = if current_timeout.is_zero() {
                    0.0
                } else {
                    (secs_left / current_timeout.as_secs_f64()).clamp(0.0, 1.0)
                };
                let cps_str = format!("{cps} cps");
                let time_str = format!("{secs_left:.1}s");
                // 8-char progress bar, color-coded by remaining fraction.
                const BAR_LEN: usize = 8;
                let available = (fraction * BAR_LEN as f64).round() as usize;
                let consumed = BAR_LEN - available;
                let consumed_part = "·".repeat(consumed).black();
                let available_part = if fraction > 0.25 {
                    "·".repeat(available).light_green()
                } else if fraction > 0.05 {
                    "·".repeat(available).yellow()
                } else {
                    "·".repeat(available).red()
                };
                let bar = format!("{available_part}{consumed_part}");
                let style = if uname.as_str() == player.uname.as_str() {
                    hud_active()
                } else {
                    hud_text()
                };

                ui::container().flex().flex_row().with_children([
                    ui::text(pname.as_str(), style.align_left()).w(15.0).h(5.0),
                    ui::text(cps_str, style.align_right()).w(8.0).h(5.0),
                    ui::text(time_str, style.align_right()).w(10.0).h(5.0),
                    ui::text(bar, style).w(10.0).h(5.0),
                ])
            })
            .collect();

        let leaderboard_rows: Vec<ui::Node<BombMsg>> = global
            .leaderboard
            .iter()
            .take(7)
            .enumerate()
            .map(|(i, (uname, pname, cps, ms))| {
                let style = if uname.as_str() == player.uname.as_str() {
                    hud_active()
                } else {
                    hud_text()
                };
                let rank = format!("#{}", i + 1);
                let cps_str = format!("{cps} cps");
                let survival_str = format!("{:.1}s", *ms as f64 / 1000.0);
                ui::container().flex().flex_row().with_children([
                    ui::text(rank, style).w(5.0).h(5.0),
                    ui::text(pname.as_str(), style.align_left()).w(20.0).h(5.0),
                    ui::text(cps_str, style.align_right()).w(8.0).h(5.0),
                    ui::text(survival_str, style.align_right()).w(10.0).h(5.0),
                ])
            })
            .collect();

        let scoreboard = ui::container()
            .flex()
            .pl(5.0)
            .w(200.0)
            .mt(10.0)
            .flex_col()
            .items_start()
            .with_child(ui::text("Active Runs", hud_title()).w(43.0).h(5.0))
            .with_children(active_run_rows)
            .with_child(ui::text("Session Best", hud_title()).w(43.0).h(5.0))
            .with_children(leaderboard_rows);

        ui::container()
            .flex()
            .flex_col()
            .with_child(
                topbar(&format!("Bomb - {}", global.phase))
                    .with_child(ui::text(status_str, status_style).w(15.0).h(5.0)),
            )
            .with_child(scoreboard)
    }
}

type BombUi = Ui<BombView, BombGlobal, BombConnectionProps>;

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

fn start_setup(state: &State<Bomb>, game: &Game, sender: &Sender, ui: &BombUi) {
    let (track, layout, setup_timeout, setup_cancel) = {
        let mut b = state.write();
        b.phase = BombPhase::SettingUp;
        let token = b.make_setup_cancel();
        (
            b.config.track,
            b.config.layout.clone(),
            b.config.setup_timeout,
            token,
        )
    };

    let _ = sender.packets(mtc(
        "Bomb - setting up track, hit /ready when prompted.",
        Some(ConnectionId::ALL),
    ));
    refresh_ui(state, ui);

    let game = game.clone();
    let sender = sender.clone();
    // Fire-and-forget: the spawn's only side effect is a synthetic event when
    // it completes; the JoinHandle is intentionally dropped.
    drop(tokio::spawn(async move {
        let result = tokio::select! {
            r = track_rotation(&game, track, RaceLaps::Untimed, 0, layout, setup_cancel.clone(), &sender) => r,
            _ = tokio::time::sleep(setup_timeout) => None,
        };
        match result {
            Some(()) => {
                let _ = sender.event(SetupComplete);
            },
            None => {
                let _ = sender.event(SetupAborted);
            },
        }
    }));
}

fn refresh_ui(state: &State<Bomb>, ui: &BombUi) {
    let snapshot = state.read().snapshot();
    ui.assign_global(snapshot);
}

async fn on_connected(
    Event(Connected(info)): Event<Connected>,
    state: State<Bomb>,
    presence: Presence,
    ui: BombUi,
    game: Game,
    sender: Sender,
) -> Result<(), AppError> {
    let _ = ui
        .assign_player(
            info.ucid,
            BombConnectionProps {
                uname: info.uname.clone(),
                in_run: false,
            },
        )
        .await;
    let should_start = {
        let b = state.read();
        b.phase == BombPhase::Waiting && presence.count() >= MIN_PLAYERS
    };
    if !should_start {
        refresh_ui(&state, &ui);
        return Ok(());
    }
    start_setup(&state, &game, &sender, &ui);
    Ok(())
}

async fn on_disconnected(
    _: Event<Disconnected>,
    state: State<Bomb>,
    presence: Presence,
    sender: Sender,
    ui: BombUi,
    clearer: PenaltyClearer,
) -> Result<(), AppError> {
    if presence.count() >= MIN_PLAYERS {
        refresh_ui(&state, &ui);
        return Ok(());
    }
    let phase = state.read().phase;
    match phase {
        BombPhase::Waiting => {
            refresh_ui(&state, &ui);
        },
        BombPhase::SettingUp => {
            // Cancel the in-flight setup; the spawn will emit SetupAborted
            // which `on_setup_aborted` translates into Waiting + retry.
            state.write().cancel_setup();
            refresh_ui(&state, &ui);
        },
        BombPhase::Racing => {
            // End the round: finalize every active run and reset.
            let now = Instant::now();
            let runs: Vec<ActiveRun> = {
                let mut b = state.write();
                b.players.clear();
                b.active_runs.drain().map(|(_, r)| r).collect()
            };
            for run in &runs {
                let _ = sender.packets(mtc(
                    format!("Run ended - left race after {} cps", run.checkpoints).red(),
                    Some(run.ucid),
                ));
                let _ = ui
                    .assign_player(
                        run.ucid,
                        BombConnectionProps {
                            uname: run.uname.clone(),
                            in_run: false,
                        },
                    )
                    .await;
            }
            {
                let mut b = state.write();
                for run in &runs {
                    let survival_ms = run.survival_ms(now);
                    b.finalize(run, survival_ms);
                }
                b.phase = BombPhase::Waiting;
            }
            clearer.clear();
            sender.packets(mtc(
                "Bomb - not enough players, restarting.",
                Some(ConnectionId::ALL),
            ))?;
            refresh_ui(&state, &ui);
        },
    }
    Ok(())
}

async fn on_setup_complete(
    _: Event<SetupComplete>,
    state: State<Bomb>,
    sender: Sender,
    ui: BombUi,
) -> Result<(), AppError> {
    let cfg_secs = {
        let mut b = state.write();
        if b.phase != BombPhase::SettingUp {
            return Ok(());
        }
        b.phase = BombPhase::Racing;
        b.clear_setup_cancel();
        b.config.checkpoint_timeout.as_secs_f64()
    };

    sender.packets(mtc(
        format!("Bomb - hit checkpoints before the {cfg_secs:.0}s timer expires!"),
        Some(ConnectionId::ALL),
    ))?;
    // Ask LFS to resend Npl for every player currently in the race so the
    // bomb players map repopulates without waiting for the next join.
    sender.packet(insim::Packet::Tiny(Tiny {
        subt: TinyType::Npl,
        ..Default::default()
    }))?;
    refresh_ui(&state, &ui);
    Ok(())
}

async fn on_setup_aborted(
    _: Event<SetupAborted>,
    state: State<Bomb>,
    presence: Presence,
    ui: BombUi,
    game: Game,
    sender: Sender,
) -> Result<(), AppError> {
    {
        let mut b = state.write();
        if b.phase != BombPhase::SettingUp {
            return Ok(());
        }
        b.phase = BombPhase::Waiting;
        b.clear_setup_cancel();
    }
    sender.packets(mtc(
        "Bomb - setup failed, restarting.",
        Some(ConnectionId::ALL),
    ))?;
    refresh_ui(&state, &ui);
    // Auto-retry if we still have the players.
    if presence.count() >= MIN_PLAYERS {
        start_setup(&state, &game, &sender, &ui);
    }
    Ok(())
}

async fn on_race_ended(
    _: Event<RaceEnded>,
    state: State<Bomb>,
    presence: Presence,
    ui: BombUi,
    game: Game,
    sender: Sender,
    clearer: PenaltyClearer,
) -> Result<(), AppError> {
    let runs: Vec<ActiveRun> = {
        let mut b = state.write();
        if b.phase != BombPhase::Racing {
            return Ok(());
        }
        b.players.clear();
        b.active_runs.drain().map(|(_, r)| r).collect()
    };
    let now = Instant::now();
    for run in &runs {
        let _ = sender.packets(mtc(
            format!("Run ended - race finished after {} cps", run.checkpoints).yellow(),
            Some(run.ucid),
        ));
        let _ = ui
            .assign_player(
                run.ucid,
                BombConnectionProps {
                    uname: run.uname.clone(),
                    in_run: false,
                },
            )
            .await;
    }
    {
        let mut b = state.write();
        for run in &runs {
            let survival_ms = run.survival_ms(now);
            b.finalize(run, survival_ms);
        }
        b.phase = BombPhase::Waiting;
    }
    clearer.clear();
    refresh_ui(&state, &ui);
    if presence.count() >= MIN_PLAYERS {
        start_setup(&state, &game, &sender, &ui);
    }
    Ok(())
}

async fn on_npl(
    Packet(npl): Packet<Npl>,
    state: State<Bomb>,
    presence: Presence,
) -> Result<(), AppError> {
    let uname = presence.get(npl.ucid).map(|c| c.uname).unwrap_or_default();
    let _ = state.write().players.insert(
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

async fn on_player_left(
    Event(PlayerLeft(player)): Event<PlayerLeft>,
    state: State<Bomb>,
    sender: Sender,
    ui: BombUi,
) -> Result<(), AppError> {
    let now = Instant::now();
    let run = {
        let mut b = state.write();
        let _ = b.players.remove(&player.plid);
        b.active_runs.remove(&player.plid)
    };
    if let Some(run) = run {
        let survival_ms = run.survival_ms(now);
        sender.packets(mtc(
            format!("Run ended - left race after {} cps", run.checkpoints).red(),
            Some(run.ucid),
        ))?;
        state.write().finalize(&run, survival_ms);
        let _ = ui
            .assign_player(
                run.ucid,
                BombConnectionProps {
                    uname: run.uname.clone(),
                    in_run: false,
                },
            )
            .await;
        refresh_ui(&state, &ui);
    }
    Ok(())
}

async fn on_toc(Packet(toc): Packet<Toc>, state: State<Bomb>) -> Result<(), AppError> {
    let mut b = state.write();
    if let Some(p) = b.players.get_mut(&toc.plid) {
        p.ucid = toc.newucid;
    }
    if let Some(r) = b.active_runs.get_mut(&toc.plid) {
        r.ucid = toc.newucid;
    }
    Ok(())
}

async fn on_pit(
    Packet(pit): Packet<Pit>,
    state: State<Bomb>,
    sender: Sender,
    ui: BombUi,
) -> Result<(), AppError> {
    let now = Instant::now();
    let run = state.write().active_runs.remove(&pit.plid);
    if let Some(run) = run {
        let survival_ms = run.survival_ms(now);
        sender.packets(mtc(
            format!(
                "PITTED - run ended after {} cps. Commit to your fuel.",
                run.checkpoints
            )
            .red(),
            Some(run.ucid),
        ))?;
        state.write().finalize(&run, survival_ms);
        let _ = ui
            .assign_player(
                run.ucid,
                BombConnectionProps {
                    uname: run.uname.clone(),
                    in_run: false,
                },
            )
            .await;
        refresh_ui(&state, &ui);
    }
    Ok(())
}

async fn on_player_teleported_to_pits(
    Event(PlayerTeleportedToPits(player)): Event<PlayerTeleportedToPits>,
    state: State<Bomb>,
    sender: Sender,
    ui: BombUi,
) -> Result<(), AppError> {
    let now = Instant::now();
    let run = state.write().active_runs.remove(&player.plid);
    if let Some(run) = run {
        let survival_ms = run.survival_ms(now);
        sender.packets(mtc(
            format!(
                "TELE-PITTED - run ended after {} cps. No shortcuts.",
                run.checkpoints
            )
            .red(),
            Some(run.ucid),
        ))?;
        state.write().finalize(&run, survival_ms);
        let _ = ui
            .assign_player(
                run.ucid,
                BombConnectionProps {
                    uname: run.uname.clone(),
                    in_run: false,
                },
            )
            .await;
        refresh_ui(&state, &ui);
    }
    Ok(())
}

async fn on_crs(
    Packet(crs): Packet<Crs>,
    state: State<Bomb>,
    sender: Sender,
    ui: BombUi,
) -> Result<(), AppError> {
    let now = Instant::now();
    let result = state.write().on_reset(crs.plid, now);
    if let Some((ucid, penalty, time_left)) = result {
        sender.packets(mtc(
            format!(
                "PENALTY -{:.2}s - {:.1}s left",
                penalty.as_secs_f64(),
                time_left.as_secs_f64()
            )
            .red(),
            Some(ucid),
        ))?;
        refresh_ui(&state, &ui);
    }
    Ok(())
}

async fn on_con(
    Packet(con): Packet<Con>,
    state: State<Bomb>,
    sender: Sender,
    ui: BombUi,
) -> Result<(), AppError> {
    let now = Instant::now();
    let mps = con.spclose.to_meters_per_sec();
    let mut any_hit = false;
    for plid in [con.a.plid, con.b.plid] {
        let result = state.write().on_collision(plid, mps, now);
        if let Some((ucid, penalty, time_left)) = result {
            sender.packets(mtc(
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
        refresh_ui(&state, &ui);
    }
    Ok(())
}

async fn on_uco(
    Packet(uco): Packet<Uco>,
    state: State<Bomb>,
    sender: Sender,
    ui: BombUi,
) -> Result<(), AppError> {
    let ObjectInfo::InsimCheckpoint(InsimCheckpoint { kind, .. }) = uco.info else {
        return Ok(());
    };
    let is_finish = matches!(kind, InsimCheckpointKind::Finish);
    let now = Instant::now();

    let outcome = {
        let mut b = state.write();
        let player = match b.players.get(&uco.plid).cloned() {
            Some(p) => p,
            None => return Ok(()),
        };
        b.on_checkpoint(&player, uco.plid, is_finish, now)
    };
    let Some(outcome) = outcome else {
        return Ok(());
    };

    match outcome {
        CheckpointOutcome::Started { ucid, uname } => {
            sender.packets(mtc(
                "Run started - hit every checkpoint!".light_green(),
                Some(ucid),
            ))?;
            let _ = ui
                .assign_player(
                    ucid,
                    BombConnectionProps {
                        uname,
                        in_run: true,
                    },
                )
                .await;
        },
        CheckpointOutcome::Refreshed {
            ucid,
            checkpoints,
            new_window,
        } => {
            sender.packets(mtc(
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
            sender.packets(mtc(
                format!("cp {checkpoints} - {:.1}s left", time_left.as_secs_f64()).light_green(),
                Some(ucid),
            ))?;
        },
    }
    refresh_ui(&state, &ui);
    Ok(())
}

async fn on_tick(
    _: Event<BombTick>,
    state: State<Bomb>,
    sender: Sender,
    ui: BombUi,
) -> Result<(), AppError> {
    let now = Instant::now();
    let expired = state.write().tick_expired(now);
    let had_expired = !expired.is_empty();
    for run in &expired {
        let survival_ms = run.survival_ms(now);
        let _ = sender.packets(mtc(
            format!(
                "BOOM - {} cps, {:.1}s",
                run.checkpoints,
                survival_ms as f64 / 1000.0
            )
            .red(),
            Some(run.ucid),
        ));
        state.write().finalize(run, survival_ms);
        let _ = ui
            .assign_player(
                run.ucid,
                BombConnectionProps {
                    uname: run.uname.clone(),
                    in_run: false,
                },
            )
            .await;
    }
    let has_active = !had_expired && !state.read().active_runs.is_empty();
    if had_expired || has_active {
        refresh_ui(&state, &ui);
    }
    Ok(())
}

#[derive(Parser, Debug)]
#[command(about = "kitcar - bomb game example (handler-driven)")]
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

    let app = App::<Bomb>::with_state(Bomb::new(config, CancellationToken::new()));
    let sender = app.sender().clone();

    // Wire the runtime's cancel token into the state so setup spawns die when
    // the runtime dies. (The runtime token is minted inside `with_state`, so
    // it can't be passed into `Bomb::new` ahead of time.)
    app.state().write().runtime_cancel = app.cancel_token().clone();

    let ui = Ui::<BombView, BombGlobal, BombConnectionProps>::new(
        sender.clone(),
        BombGlobal {
            phase: BombPhase::Waiting.label().to_string(),
            ..Default::default()
        },
        |_ucid, invalidator| {
            // Force a redraw every 100ms so the active-run timer counts down
            // smoothly even when no underlying state change has fired.
            let _tick_handle = tokio::spawn(async move {
                let mut interval = tokio::time::interval(Duration::from_millis(100));
                loop {
                    let _ = interval.tick().await;
                    invalidator.invalidate();
                }
            });
            BombView { _tick_handle }
        },
    );

    let presence = Presence::new();
    let game = Game::new();
    let clearer = PenaltyClearer::new(PENALTY_CLEAR_DELAY);

    // Predicate that gates in-round handlers. Reads `phase` directly from the
    // app's state.
    let while_racing = |s: State<Bomb>| s.read().phase == BombPhase::Racing;

    let app = app
        .handle(Stage::Pre, presence)
        .handle(Stage::Pre, game)
        .handle(Stage::Pre, clearer)
        .handle(Stage::Pre, ui)
        // Phase transitions.
        .handle(Stage::Update, on_connected)
        .handle(Stage::Update, on_disconnected)
        .handle(Stage::Update, on_setup_complete)
        .handle(Stage::Update, on_setup_aborted)
        .handle(Stage::Update, on_race_ended)
        // In-round handlers - only run while we're racing.
        .handle(Stage::Update, on_npl.run_if(while_racing))
        .handle(Stage::Update, on_player_left.run_if(while_racing))
        .handle(Stage::Update, on_toc.run_if(while_racing))
        .handle(Stage::Update, on_pit.run_if(while_racing))
        .handle(
            Stage::Update,
            on_player_teleported_to_pits.run_if(while_racing),
        )
        .handle(Stage::Update, on_crs.run_if(while_racing))
        .handle(Stage::Update, on_con.run_if(while_racing))
        .handle(Stage::Update, on_uco.run_if(while_racing))
        .handle(Stage::Update, on_tick.run_if(while_racing))
        // Background ticker emitting BombTick every TICK_PERIOD.
        .periodic(TICK_PERIOD, BombTick);

    let builder = insim::tcp(args.addr)
        .isi_iname("bomb".to_string())
        .isi_prefix('!')
        .isi_admin_password(args.admin_password);

    run(builder, app).await
}
