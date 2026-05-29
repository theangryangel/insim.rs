use std::time::Instant;

use insim::{
    Colour,
    core::object::{
        ObjectInfo,
        insim::{InsimCheckpoint, InsimCheckpointKind},
    },
    identifiers::ConnectionId,
    insim::{Con, Crs, Pit, RaceLaps, Toc, Uco},
};
use kitcar::{
    AppError, Connected, Disconnected, Event, Game, Packet, PenaltyClearer, PlayerLeft,
    PlayerTeleportedToPits, Presence, RaceEnded, Sender, State, track_rotation, util::mtc,
};

use super::{
    config::MIN_PLAYERS,
    db::persist_bomb_run,
    events::{BombTick, SetupAborted, SetupComplete},
    state::{ActiveRun, Bomb, BombPhase, CheckpointOutcome},
    ui::{BombConnectionProps, BombUi},
};

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

    refresh_ui(state, ui);

    let game = game.clone();
    let sender = sender.clone();
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

pub(super) async fn on_connected(
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

pub(super) async fn on_disconnected(
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
            state.write().cancel_setup();
            refresh_ui(&state, &ui);
        },
        BombPhase::Racing => {
            let now = Instant::now();
            let runs: Vec<ActiveRun> = state.write().active_runs.drain().map(|(_, r)| r).collect();
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
            let db = state.read().db.clone();
            let finalized: Vec<(ActiveRun, i64)> = {
                let mut b = state.write();
                let result: Vec<(ActiveRun, i64)> = runs
                    .iter()
                    .map(|run| {
                        let survival_ms = run.survival_ms(now);
                        b.finalize(run, survival_ms);
                        (run.clone(), survival_ms)
                    })
                    .collect();
                b.phase = BombPhase::Waiting;
                result
            };
            for (run, survival_ms) in &finalized {
                persist_bomb_run(&db, run, *survival_ms).await;
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

pub(super) async fn on_setup_complete(
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
    refresh_ui(&state, &ui);
    Ok(())
}

pub(super) async fn on_setup_aborted(
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
    if presence.count() >= MIN_PLAYERS {
        start_setup(&state, &game, &sender, &ui);
    }
    Ok(())
}

pub(super) async fn on_race_ended(
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
    let db = state.read().db.clone();
    let finalized: Vec<(ActiveRun, i64)> = {
        let mut b = state.write();
        let result: Vec<(ActiveRun, i64)> = runs
            .iter()
            .map(|run| {
                let survival_ms = run.survival_ms(now);
                b.finalize(run, survival_ms);
                (run.clone(), survival_ms)
            })
            .collect();
        b.phase = BombPhase::Waiting;
        result
    };
    for (run, survival_ms) in &finalized {
        persist_bomb_run(&db, run, *survival_ms).await;
    }
    clearer.clear();
    refresh_ui(&state, &ui);
    if presence.count() >= MIN_PLAYERS {
        start_setup(&state, &game, &sender, &ui);
    }
    Ok(())
}

pub(super) async fn on_player_left(
    Event(PlayerLeft(player)): Event<PlayerLeft>,
    state: State<Bomb>,
    sender: Sender,
    ui: BombUi,
) -> Result<(), AppError> {
    let now = Instant::now();
    let (run, db) = {
        let mut b = state.write();
        let run = b.active_runs.remove(&player.plid);
        let db = b.db.clone();
        (run, db)
    };
    if let Some(run) = run {
        let survival_ms = run.survival_ms(now);
        sender.packets(mtc(
            format!("Run ended - left race after {} cps", run.checkpoints).red(),
            Some(run.ucid),
        ))?;
        persist_bomb_run(&db, &run, survival_ms).await;
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

pub(super) async fn on_toc(Packet(toc): Packet<Toc>, state: State<Bomb>) -> Result<(), AppError> {
    if let Some(r) = state.write().active_runs.get_mut(&toc.plid) {
        r.ucid = toc.newucid;
    }
    Ok(())
}

pub(super) async fn on_pit(
    Packet(pit): Packet<Pit>,
    state: State<Bomb>,
    sender: Sender,
    ui: BombUi,
) -> Result<(), AppError> {
    let now = Instant::now();
    let (run, db) = {
        let mut b = state.write();
        let run = b.active_runs.remove(&pit.plid);
        let db = b.db.clone();
        (run, db)
    };
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
        persist_bomb_run(&db, &run, survival_ms).await;
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

pub(super) async fn on_player_teleported_to_pits(
    Event(PlayerTeleportedToPits(player)): Event<PlayerTeleportedToPits>,
    state: State<Bomb>,
    sender: Sender,
    ui: BombUi,
) -> Result<(), AppError> {
    let now = Instant::now();
    let (run, db) = {
        let mut b = state.write();
        let run = b.active_runs.remove(&player.plid);
        let db = b.db.clone();
        (run, db)
    };
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
        persist_bomb_run(&db, &run, survival_ms).await;
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

pub(super) async fn on_crs(
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

pub(super) async fn on_con(
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

pub(super) async fn on_uco(
    Packet(uco): Packet<Uco>,
    state: State<Bomb>,
    presence: Presence,
    sender: Sender,
    ui: BombUi,
) -> Result<(), AppError> {
    let ObjectInfo::InsimCheckpoint(InsimCheckpoint { kind, .. }) = uco.info else {
        return Ok(());
    };
    let is_finish = matches!(kind, InsimCheckpointKind::Finish);
    let now = Instant::now();

    let Some(player) = presence.player(uco.plid) else {
        return Ok(());
    };
    let uname = presence
        .get(player.ucid)
        .map(|c| c.uname)
        .unwrap_or_default();

    let outcome = state.write().on_checkpoint(
        uco.plid,
        player.ucid,
        &uname,
        &player.pname,
        player.ptype,
        is_finish,
        now,
    );
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

pub(super) async fn on_tick(
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
        let db = state.read().db.clone();
        persist_bomb_run(&db, run, survival_ms).await;
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
