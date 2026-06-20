use insim::{
    Colour,
    core::object::{
        ObjectInfo,
        insim::{InsimCheckpoint, InsimCheckpointKind},
    },
    identifiers::ConnectionId,
    insim::{PlayerType, Toc, Uco},
};
use kitcar::{
    AppError, Connected, Disconnected, Event, Packet, RoundEndReason, RoundEnded, RoundManager,
    RoundStarted, Sender, State, World, mtc,
};

use super::{
    state::Shortcut,
    ui::{ShortcutConnectionProps, ShortcutUi},
};

fn refresh_ui(state: &State<Shortcut>, ui: &ShortcutUi) {
    let snapshot = state.read().snapshot();
    ui.assign_global(snapshot);
}

pub(super) async fn on_connected(
    Event(Connected(info)): Event<Connected>,
    state: State<Shortcut>,
    ui: ShortcutUi,
    rounds: RoundManager,
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
    state.write().phase = rounds.phase();
    refresh_ui(&state, &ui);
    Ok(())
}

pub(super) async fn on_disconnected(
    _: Event<Disconnected>,
    state: State<Shortcut>,
    ui: ShortcutUi,
    rounds: RoundManager,
) -> Result<(), AppError> {
    state.write().phase = rounds.phase();
    refresh_ui(&state, &ui);
    Ok(())
}

pub(super) async fn on_round_started(
    _: Event<RoundStarted>,
    state: State<Shortcut>,
    sender: Sender,
    ui: ShortcutUi,
    rounds: RoundManager,
) -> Result<(), AppError> {
    state.write().phase = rounds.phase();
    sender.packets(mtc(
        "Shortcut - cross checkpoint 1 to start your timed attempt!",
        Some(ConnectionId::ALL),
    ))?;
    refresh_ui(&state, &ui);
    Ok(())
}

pub(super) async fn on_round_ended(
    Event(RoundEnded(reason)): Event<RoundEnded>,
    state: State<Shortcut>,
    ui: ShortcutUi,
    sender: Sender,
    rounds: RoundManager,
) -> Result<(), AppError> {
    {
        let mut s = state.write();
        s.active_runs.clear();
        s.phase = rounds.phase();
    }
    if matches!(reason, RoundEndReason::NotEnoughPlayers) {
        let _ = sender.packets(mtc(
            "Shortcut - not enough players, restarting.",
            Some(ConnectionId::ALL),
        ));
    }
    refresh_ui(&state, &ui);
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
    let uname = world
        .connection(player.ucid)
        .map(|c| c.uname)
        .unwrap_or_default();

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

            if let Some(pkt) = world.connection(player.ucid).map(|c| c.spec()) {
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
