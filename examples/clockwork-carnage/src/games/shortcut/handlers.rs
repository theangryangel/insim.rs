use insim::{
    Colour,
    core::object::{
        ObjectInfo,
        insim::{InsimCheckpoint, InsimCheckpointKind},
    },
    identifiers::ConnectionId,
    insim::{PlayerType, RaceLaps, Toc, Uco},
};
use kitcar::{
    AppError, Connected, Disconnected, Event, Packet, Sender, SessionEnded, State, World,
    track_rotation, util::mtc,
};

use super::{
    config::MIN_PLAYERS,
    events::{SetupAborted, SetupComplete},
    state::{Shortcut, ShortcutPhase},
    ui::{ShortcutConnectionProps, ShortcutUi},
};

fn start_setup(state: &State<Shortcut>, world: &World, sender: &Sender, ui: &ShortcutUi) {
    let (track, layout, setup_timeout, setup_cancel) = {
        let mut s = state.write();
        s.phase = ShortcutPhase::SettingUp;
        let token = s.make_setup_cancel();
        (
            s.config.track,
            s.config.layout.clone(),
            s.config.setup_timeout,
            token,
        )
    };

    let _ = sender.packets(mtc(
        "Shortcut - setting up track, hit /ready when prompted.",
        Some(ConnectionId::ALL),
    ));
    refresh_ui(state, ui);

    let world = world.clone();
    let sender = sender.clone();
    drop(tokio::spawn(async move {
        let result = tokio::select! {
            r = track_rotation(&world, track, RaceLaps::Untimed, 0, layout, setup_cancel.clone(), &sender) => r,
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

fn refresh_ui(state: &State<Shortcut>, ui: &ShortcutUi) {
    let snapshot = state.read().snapshot();
    ui.assign_global(snapshot);
}

pub(super) async fn on_connected(
    Event(Connected(info)): Event<Connected>,
    state: State<Shortcut>,
    world: World,
    ui: ShortcutUi,
    sender: Sender,
) -> Result<(), AppError> {
    let _ = ui
        .assign_player(
            info.ucid,
            ShortcutConnectionProps {
                uname: info.uname.clone(),
                in_run: false,
                best_time_ms: None,
            },
        )
        .await;
    let should_start = {
        let s = state.read();
        s.phase == ShortcutPhase::Waiting && world.count() >= MIN_PLAYERS
    };
    if !should_start {
        refresh_ui(&state, &ui);
        return Ok(());
    }
    start_setup(&state, &world, &sender, &ui);
    Ok(())
}

pub(super) async fn on_disconnected(
    _: Event<Disconnected>,
    state: State<Shortcut>,
    world: World,
    sender: Sender,
    ui: ShortcutUi,
) -> Result<(), AppError> {
    if world.count() >= MIN_PLAYERS {
        refresh_ui(&state, &ui);
        return Ok(());
    }
    let phase = state.read().phase;
    match phase {
        ShortcutPhase::Waiting => {
            refresh_ui(&state, &ui);
        },
        ShortcutPhase::SettingUp => {
            state.write().cancel_setup();
            refresh_ui(&state, &ui);
        },
        ShortcutPhase::Racing => {
            {
                let mut s = state.write();
                s.active_runs.clear();
                s.phase = ShortcutPhase::Waiting;
            }
            sender.packets(mtc(
                "Shortcut - not enough players, restarting.",
                Some(ConnectionId::ALL),
            ))?;
            refresh_ui(&state, &ui);
        },
    }
    Ok(())
}

pub(super) async fn on_setup_complete(
    _: Event<SetupComplete>,
    state: State<Shortcut>,
    sender: Sender,
    ui: ShortcutUi,
) -> Result<(), AppError> {
    {
        let mut s = state.write();
        if s.phase != ShortcutPhase::SettingUp {
            return Ok(());
        }
        s.phase = ShortcutPhase::Racing;
        s.clear_setup_cancel();
    }
    sender.packets(mtc(
        "Shortcut - cross checkpoint 1 to start your timed attempt!",
        Some(ConnectionId::ALL),
    ))?;
    refresh_ui(&state, &ui);
    Ok(())
}

pub(super) async fn on_setup_aborted(
    _: Event<SetupAborted>,
    state: State<Shortcut>,
    world: World,
    ui: ShortcutUi,
    sender: Sender,
) -> Result<(), AppError> {
    {
        let mut s = state.write();
        if s.phase != ShortcutPhase::SettingUp {
            return Ok(());
        }
        s.phase = ShortcutPhase::Waiting;
        s.clear_setup_cancel();
    }
    sender.packets(mtc(
        "Shortcut - setup failed, restarting.",
        Some(ConnectionId::ALL),
    ))?;
    refresh_ui(&state, &ui);
    if world.count() >= MIN_PLAYERS {
        start_setup(&state, &world, &sender, &ui);
    }
    Ok(())
}

pub(super) async fn on_race_ended(
    _: Event<SessionEnded>,
    state: State<Shortcut>,
    world: World,
    ui: ShortcutUi,
    sender: Sender,
) -> Result<(), AppError> {
    {
        let mut s = state.write();
        if s.phase != ShortcutPhase::Racing {
            return Ok(());
        }
        s.active_runs.clear();
        s.phase = ShortcutPhase::Waiting;
    }
    refresh_ui(&state, &ui);
    if world.count() >= MIN_PLAYERS {
        start_setup(&state, &world, &sender, &ui);
    }
    Ok(())
}

pub(super) async fn on_toc(
    Packet(toc): Packet<Toc>,
    state: State<Shortcut>,
) -> Result<(), AppError> {
    if let Some(r) = state.write().active_runs.get_mut(&toc.plid) {
        r.0 = toc.newucid;
    }
    Ok(())
}

pub(super) async fn on_uco(
    Packet(uco): Packet<Uco>,
    state: State<Shortcut>,
    sender: Sender,
    ui: ShortcutUi,
    world: World,
) -> Result<(), AppError> {
    let ObjectInfo::InsimCheckpoint(InsimCheckpoint { kind, .. }) = uco.info else {
        return Ok(());
    };
    let is_cp1 = matches!(kind, InsimCheckpointKind::Checkpoint1);
    let is_finish = matches!(kind, InsimCheckpointKind::Finish);
    if !is_cp1 && !is_finish {
        return Ok(());
    }

    let Some(player) = world.player(uco.plid) else {
        return Ok(());
    };
    if player.ptype.contains(PlayerType::AI) {
        return Ok(());
    }
    let uname = world.get(player.ucid).map(|c| c.uname).unwrap_or_default();

    if is_cp1 {
        let _ = state
            .write()
            .active_runs
            .insert(uco.plid, (player.ucid, uname.clone(), uco.time));
        sender.packets(mtc(
            "Run started - reach the finish!".light_green(),
            Some(player.ucid),
        ))?;
        let _ = ui
            .assign_player(
                player.ucid,
                ShortcutConnectionProps {
                    uname: uname.clone(),
                    in_run: true,
                    best_time_ms: None,
                },
            )
            .await;
    } else if is_finish {
        let run = state.write().active_runs.remove(&uco.plid);
        if let Some((_ucid, uname, start_time)) = run {
            let lap_time = uco.time.saturating_sub(start_time);
            let time_ms = lap_time.as_millis() as i64;
            let vehicle = player.vehicle.to_string();

            if let Some(pkt) = world.spec(player.ucid) {
                let _ = sender.packet(pkt);
            }

            let prev_best = state
                .read()
                .leaderboard
                .iter()
                .find(|e| e.0 == uname)
                .map(|e| e.2);
            let is_pb = match prev_best {
                Some(prev) => time_ms < prev,
                None => true,
            };

            let db = state.read().db.clone();
            if let Some((pool, event_id)) = &db {
                if let Err(e) =
                    crate::db::insert_shortcut_time(pool, *event_id, &uname, &vehicle, time_ms)
                        .await
                {
                    tracing::warn!("Failed to persist shortcut time: {e}");
                }
            }

            state
                .write()
                .update_leaderboard(&uname, &player.pname, time_ms);

            let mins = time_ms / 60_000;
            let secs = (time_ms % 60_000) / 1000;
            let millis = time_ms % 1000;
            let time_str = format!("{mins}:{secs:02}.{millis:03}");

            if is_pb {
                sender.packets(mtc(
                    format!("New PB! {} ({})", time_str, vehicle).light_green(),
                    Some(player.ucid),
                ))?;
            } else if let Some(prev) = prev_best {
                let prev_mins = prev / 60_000;
                let prev_secs = (prev % 60_000) / 1000;
                let prev_millis = prev % 1000;
                sender.packets(mtc(
                    format!(
                        "Time: {} | PB: {}:{:02}.{:03}",
                        time_str, prev_mins, prev_secs, prev_millis
                    )
                    .yellow(),
                    Some(player.ucid),
                ))?;
            }

            sender.packets(mtc("Rejoin to retry".yellow(), Some(player.ucid)))?;

            let best_in_lb = state
                .read()
                .leaderboard
                .iter()
                .find(|e| e.0 == uname)
                .map(|e| e.2);

            let _ = ui
                .assign_player(
                    player.ucid,
                    ShortcutConnectionProps {
                        uname: uname.clone(),
                        in_run: false,
                        best_time_ms: best_in_lb,
                    },
                )
                .await;
        }
    }

    refresh_ui(&state, &ui);
    Ok(())
}
