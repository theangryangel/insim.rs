use std::{cmp::Reverse, time::Instant};

use insim::{
    Colour,
    core::object::{
        ObjectInfo,
        insim::{InsimCheckpoint, InsimCircle},
    },
    identifiers::ConnectionId,
    insim::{Axm, Con, Crs, Pit, PmoAction, PmoFlags, Uco, UcoAction},
};
use kitcar::{
    AppError, Connected, Disconnected, Event, Packet, PenaltyClearer, RoundEndReason, RoundEnded,
    RoundManager, RoundStarted, Sender, State, World, mtc,
};

use super::{
    db::persist_bomb_run,
    events::BombTick,
    state::{ActiveRun, Bomb, BombGlobal, CheckpointOutcome},
    ui::{BombConnectionProps, BombUi},
};
use crate::run_registry::{RunEndReason, RunEnded, RunRegistry};

/// Rebuild the global board from authoritative state: phase + leaderboard from
/// the bomb state, in-progress runs from the registry.
fn refresh_ui(state: &State<Bomb>, runs: &RunRegistry<ActiveRun>, ui: &BombUi) {
    let (phase, leaderboard) = {
        let b = state.read();
        (b.phase, b.leaderboard.clone())
    };
    let mut active: Vec<(String, String, i64, Instant, std::time::Duration)> = runs
        .snapshot()
        .into_iter()
        .map(|(_, r)| {
            (
                r.uname,
                r.pname,
                r.checkpoints,
                r.deadline,
                r.current_timeout,
            )
        })
        .collect();
    active.sort_by_key(|r| Reverse(r.2));
    ui.assign_global(BombGlobal {
        phase,
        leaderboard,
        active_runs: active,
    });
}

/// Push one player's per-connection UI props (their name + whether they're
/// mid-run).
async fn set_player(ui: &BombUi, ucid: ConnectionId, uname: String, in_run: bool) {
    let _ = ui
        .assign_player(ucid, BombConnectionProps { uname, in_run })
        .await;
}

/// Finalize an already-removed run: announce `message` to the driver (if still
/// connected), persist the result, fold it into the leaderboard, clear their
/// HUD, and refresh the board. The caller is responsible for having removed the
/// run from the registry and for resolving `ucid` from the world (`None` if the
/// player has already left).
async fn end_run(
    state: &State<Bomb>,
    runs: &RunRegistry<ActiveRun>,
    ui: &BombUi,
    sender: &Sender,
    ucid: Option<ConnectionId>,
    run: &ActiveRun,
    message: String,
) {
    let now = Instant::now();
    let survival_ms = run.survival_ms(now);
    let db = state.read().db.clone();
    if let Some(ucid) = ucid {
        let _ = sender.packets(mtc(message, Some(ucid)));
        set_player(ui, ucid, run.uname.clone(), false).await;
    }
    persist_bomb_run(&db, run, survival_ms).await;
    state.write().finalize(run, survival_ms);
    refresh_ui(state, runs, ui);
}

pub(super) async fn on_connected(
    Event(Connected(info)): Event<Connected>,
    state: State<Bomb>,
    runs: RunRegistry<ActiveRun>,
    ui: BombUi,
    rounds: RoundManager,
) -> Result<(), AppError> {
    set_player(&ui, info.ucid, info.uname.clone(), false).await;
    // RoundManager owns starting/rotation; mirror its phase for the UI.
    state.write().phase = rounds.phase();
    refresh_ui(&state, &runs, &ui);
    Ok(())
}

pub(super) async fn on_disconnected(
    _: Event<Disconnected>,
    state: State<Bomb>,
    runs: RunRegistry<ActiveRun>,
    ui: BombUi,
    rounds: RoundManager,
) -> Result<(), AppError> {
    // The registry already evicted any run the leaving player had (emitting
    // RunEnded); here we just track the phase for the UI.
    state.write().phase = rounds.phase();
    refresh_ui(&state, &runs, &ui);
    Ok(())
}

pub(super) async fn on_round_started(
    _: Event<RoundStarted>,
    state: State<Bomb>,
    runs: RunRegistry<ActiveRun>,
    sender: Sender,
    ui: BombUi,
    rounds: RoundManager,
) -> Result<(), AppError> {
    let cfg_secs = {
        let mut b = state.write();
        b.phase = rounds.phase();
        b.config.checkpoint_timeout.as_secs_f64()
    };
    sender.packets(mtc(
        format!("Bomb - hit checkpoints before the {cfg_secs:.0}s timer expires!"),
        Some(ConnectionId::ALL),
    ))?;
    refresh_ui(&state, &runs, &ui);
    Ok(())
}

#[allow(clippy::too_many_arguments)] // a handler's args are all magic extractors
pub(super) async fn on_round_ended(
    Event(RoundEnded(reason)): Event<RoundEnded>,
    state: State<Bomb>,
    runs: RunRegistry<ActiveRun>,
    world: World,
    ui: BombUi,
    sender: Sender,
    rounds: RoundManager,
    clearer: PenaltyClearer,
) -> Result<(), AppError> {
    let ended_msg = match reason {
        RoundEndReason::SessionEnded => "Run ended - race finished after",
        RoundEndReason::NotEnoughPlayers => "Run ended - not enough players after",
    };
    for (plid, run) in runs.drain_where(|_| true) {
        let ucid = world.player(plid).map(|p| p.ucid);
        let message = format!("{ended_msg} {} cps", run.checkpoints).yellow();
        end_run(&state, &runs, &ui, &sender, ucid, &run, message).await;
    }
    state.write().phase = rounds.phase();
    clearer.clear();
    if matches!(reason, RoundEndReason::NotEnoughPlayers) {
        let _ = sender.packets(mtc(
            "Bomb - not enough players, restarting.",
            Some(ConnectionId::ALL),
        ));
    }
    refresh_ui(&state, &runs, &ui);
    Ok(())
}

/// A run was auto-evicted because its owner left the track or tele-pitted.
pub(super) async fn on_run_ended(
    Event(RunEnded { plid, run, reason }): Event<RunEnded<ActiveRun>>,
    state: State<Bomb>,
    runs: RunRegistry<ActiveRun>,
    world: World,
    sender: Sender,
    ui: BombUi,
) -> Result<(), AppError> {
    let ucid = world.player(plid).map(|p| p.ucid);
    let message = match reason {
        RunEndReason::Left => format!("Run ended - left race after {} cps", run.checkpoints).red(),
        RunEndReason::TeleportedToPits => format!(
            "TELE-PITTED - run ended after {} cps. No shortcuts.",
            run.checkpoints
        )
        .red(),
    };
    end_run(&state, &runs, &ui, &sender, ucid, &run, message).await;
    Ok(())
}

pub(super) async fn on_pit(
    Packet(pit): Packet<Pit>,
    state: State<Bomb>,
    runs: RunRegistry<ActiveRun>,
    world: World,
    sender: Sender,
    ui: BombUi,
) -> Result<(), AppError> {
    if let Some(run) = runs.finish(pit.plid) {
        let ucid = world.player(pit.plid).map(|p| p.ucid);
        let message = format!(
            "PITTED - run ended after {} cps. Commit to your fuel.",
            run.checkpoints
        )
        .red();
        end_run(&state, &runs, &ui, &sender, ucid, &run, message).await;
    }
    Ok(())
}

pub(super) async fn on_crs(
    Packet(crs): Packet<Crs>,
    state: State<Bomb>,
    runs: RunRegistry<ActiveRun>,
    world: World,
    sender: Sender,
    ui: BombUi,
) -> Result<(), AppError> {
    let now = Instant::now();
    if let Some((penalty, time_left)) = state.read().on_reset(&runs, crs.plid, now) {
        if let Some(ucid) = world.player(crs.plid).map(|p| p.ucid) {
            sender.packets(mtc(
                format!(
                    "PENALTY -{:.2}s - {:.1}s left",
                    penalty.as_secs_f64(),
                    time_left.as_secs_f64()
                )
                .red(),
                Some(ucid),
            ))?;
        }
        refresh_ui(&state, &runs, &ui);
    }
    Ok(())
}

pub(super) async fn on_con(
    Packet(con): Packet<Con>,
    state: State<Bomb>,
    runs: RunRegistry<ActiveRun>,
    world: World,
    sender: Sender,
    ui: BombUi,
) -> Result<(), AppError> {
    let now = Instant::now();
    let mps = con.spclose.to_metres_per_sec();
    let mut any_hit = false;
    for plid in [con.a.plid, con.b.plid] {
        if let Some((penalty, time_left)) = state.read().on_collision(&runs, plid, mps, now) {
            if let Some(ucid) = world.player(plid).map(|p| p.ucid) {
                sender.packets(mtc(
                    format!(
                        "PENALTY -{:.2}s - {:.1}s left",
                        penalty.as_secs_f64(),
                        time_left.as_secs_f64()
                    )
                    .red(),
                    Some(ucid),
                ))?;
            }
            any_hit = true;
        }
    }
    if any_hit {
        refresh_ui(&state, &runs, &ui);
    }
    Ok(())
}

pub(super) async fn on_axm(Packet(axm): Packet<Axm>, state: State<Bomb>) -> Result<(), AppError> {
    let file_end = axm.flags.contains(PmoFlags::FILE_END);
    match axm.action {
        PmoAction::ClearAll => {
            state.write().clear_circles();
        },
        PmoAction::LoadingFile(objects) | PmoAction::TinyAxm(objects) => {
            let indices = objects.into_iter().filter_map(|o| {
                if let ObjectInfo::InsimCircle(InsimCircle { index, .. }) = o {
                    Some(index)
                } else {
                    None
                }
            });
            state.write().accumulate_circles(indices);
            if file_end {
                state.write().finalize_circles();
            }
        },
        _ => {},
    }
    Ok(())
}

pub(super) async fn on_uco(
    Packet(uco): Packet<Uco>,
    state: State<Bomb>,
    runs: RunRegistry<ActiveRun>,
    world: World,
    sender: Sender,
    ui: BombUi,
) -> Result<(), AppError> {
    let now = Instant::now();

    let outcome = match (&uco.ucoaction, &uco.info) {
        (UcoAction::CircleEnter, ObjectInfo::InsimCircle(InsimCircle { index, .. })) => {
            let Some(player) = world.player(uco.plid) else {
                return Ok(());
            };
            let uname = world
                .connection(player.ucid)
                .map(|c| c.uname)
                .unwrap_or_default();
            state
                .read()
                .on_checkpoint(&runs, &uname, &player, *index, now)
        },
        (_, ObjectInfo::InsimCheckpoint(InsimCheckpoint { .. })) => {
            state.read().on_time_bonus(&runs, uco.plid, now)
        },
        _ => return Ok(()),
    };
    let Some(outcome) = outcome else {
        return Ok(());
    };
    let Some(ucid) = world.player(uco.plid).map(|p| p.ucid) else {
        return Ok(());
    };

    match outcome {
        CheckpointOutcome::Started { uname } => {
            sender.packets(mtc(
                "Run started - hit every checkpoint!".light_green(),
                Some(ucid),
            ))?;
            set_player(&ui, ucid, uname, true).await;
        },
        CheckpointOutcome::Refreshed {
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
            checkpoints,
            time_left,
        } => {
            sender.packets(mtc(
                format!("cp {checkpoints} - {:.1}s left", time_left.as_secs_f64()).light_green(),
                Some(ucid),
            ))?;
        },
    }
    refresh_ui(&state, &runs, &ui);
    Ok(())
}

pub(super) async fn on_tick(
    _: Event<BombTick>,
    state: State<Bomb>,
    runs: RunRegistry<ActiveRun>,
    world: World,
    sender: Sender,
    ui: BombUi,
) -> Result<(), AppError> {
    let now = Instant::now();
    let expired = runs.drain_where(|r| r.deadline < now);
    let nothing_expired = expired.is_empty();
    for (plid, run) in &expired {
        let ucid = world.player(*plid).map(|p| p.ucid);
        let message = format!(
            "BOOM - {} cps, {:.1}s",
            run.checkpoints,
            run.survival_ms(now) as f64 / 1000.0
        )
        .red();
        end_run(&state, &runs, &ui, &sender, ucid, run, message).await;
    }
    // Nothing expired this tick, but active runs still need their countdown
    // rows kept fresh.
    if nothing_expired && !runs.is_empty() {
        refresh_ui(&state, &runs, &ui);
    }
    Ok(())
}
