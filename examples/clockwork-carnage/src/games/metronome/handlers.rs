use std::time::Duration;

use insim::{
    Colour,
    core::object::{
        ObjectInfo,
        insim::{InsimCheckpoint, InsimCheckpointKind},
    },
    identifiers::{ConnectionId, PlayerId},
    insim::Uco,
};
use kitcar::{
    AppError, Connected, Disconnected, Event, Packet, PlayerLeft, PlayerTeleportedToPits,
    RoundEndReason, RoundEnded, RoundStarted, Sender, State, TakingOver, World, mtc,
};

use super::{
    state::Metronome,
    ui::{MetronomeConnectionProps, MetronomeUi},
};

fn refresh_ui(state: &State<Metronome>, ui: &MetronomeUi) {
    let snapshot = state.read().snapshot();
    ui.assign_global(snapshot);
}

/// Drop a player's in-progress run without scoring it - used when they leave
/// the track or tele-pit mid-attempt (an incomplete timed run doesn't count).
/// Resetting the per-player props clears their `in_run` HUD state if they're
/// still connected.
async fn abandon_run(state: &State<Metronome>, ui: &MetronomeUi, plid: PlayerId) {
    let run = state.write().active_runs.remove(&plid);
    if let Some((ucid, uname, _start)) = run {
        let best_delta_ms = state
            .read()
            .leaderboard
            .iter()
            .find(|e| e.0 == uname)
            .map(|e| e.2);
        let _ = ui
            .assign_player(
                ucid,
                MetronomeConnectionProps {
                    uname,
                    in_run: false,
                    best_delta_ms,
                },
            )
            .await;
        refresh_ui(state, ui);
    }
}

pub(super) async fn on_connected(
    Event(Connected(info)): Event<Connected>,
    state: State<Metronome>,
    ui: MetronomeUi,
) -> Result<(), AppError> {
    let _ = ui
        .assign_player(
            info.ucid,
            MetronomeConnectionProps {
                uname: info.uname.clone(),
                in_run: false,
                best_delta_ms: None,
            },
        )
        .await;
    refresh_ui(&state, &ui);
    Ok(())
}

pub(super) async fn on_disconnected(
    _: Event<Disconnected>,
    state: State<Metronome>,
    ui: MetronomeUi,
) -> Result<(), AppError> {
    // Round start/teardown is owned by RoundManager (see on_round_ended).
    refresh_ui(&state, &ui);
    Ok(())
}

pub(super) async fn on_round_started(
    _: Event<RoundStarted>,
    state: State<Metronome>,
    sender: Sender,
    ui: MetronomeUi,
) -> Result<(), AppError> {
    let target_secs = state.read().config.target.as_secs_f64();
    sender.packets(mtc(
        format!("Metronome - target: {target_secs:.1}s. Cross checkpoint 1 to start."),
        Some(ConnectionId::ALL),
    ))?;
    refresh_ui(&state, &ui);
    Ok(())
}

pub(super) async fn on_round_ended(
    Event(RoundEnded(reason)): Event<RoundEnded>,
    state: State<Metronome>,
    ui: MetronomeUi,
    sender: Sender,
) -> Result<(), AppError> {
    state.write().active_runs.clear();
    if matches!(reason, RoundEndReason::NotEnoughPlayers) {
        let _ = sender.packets(mtc(
            "Metronome - not enough players, restarting.",
            Some(ConnectionId::ALL),
        ));
    }
    refresh_ui(&state, &ui);
    Ok(())
}

pub(super) async fn on_toc(
    Event(TakingOver { after, .. }): Event<TakingOver>,
    state: State<Metronome>,
) -> Result<(), AppError> {
    if let Some(r) = state.write().active_runs.get_mut(&after.plid) {
        r.0 = after.ucid;
    }
    Ok(())
}

pub(super) async fn on_player_left(
    Event(PlayerLeft(player)): Event<PlayerLeft>,
    state: State<Metronome>,
    ui: MetronomeUi,
) -> Result<(), AppError> {
    abandon_run(&state, &ui, player.plid).await;
    Ok(())
}

pub(super) async fn on_player_teleported_to_pits(
    Event(PlayerTeleportedToPits(player)): Event<PlayerTeleportedToPits>,
    state: State<Metronome>,
    ui: MetronomeUi,
) -> Result<(), AppError> {
    abandon_run(&state, &ui, player.plid).await;
    Ok(())
}

pub(super) async fn on_uco(
    Packet(uco): Packet<Uco>,
    state: State<Metronome>,
    sender: Sender,
    ui: MetronomeUi,
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
    if player.ptype.contains(insim::insim::PlayerType::AI) {
        return Ok(());
    }
    let uname = world
        .connection(player.ucid)
        .map(|c| c.uname)
        .unwrap_or_default();

    if is_cp1 {
        let start_time = uco.time;
        let _ = state
            .write()
            .active_runs
            .insert(uco.plid, (player.ucid, uname.clone(), start_time));
        sender.packets(mtc(
            "Run started - reach the finish!".light_green(),
            Some(player.ucid),
        ))?;
        let _ = ui
            .assign_player(
                player.ucid,
                MetronomeConnectionProps {
                    uname: uname.clone(),
                    in_run: true,
                    best_delta_ms: None,
                },
            )
            .await;
    } else if is_finish {
        let run = state.write().active_runs.remove(&uco.plid);
        if let Some((_ucid, uname, start_time)) = run {
            let elapsed = uco.time.saturating_sub(start_time);
            let target = state.read().config.target;
            let delta = target.abs_diff(elapsed);
            let delta_ms = delta.as_millis() as i64;

            if let Some(pkt) = world.connection(player.ucid).map(|c| c.spec()) {
                let _ = sender.packet(pkt);
            }

            let db = state.read().db.clone();
            if let Some((pool, event_id)) = &db
                && let Err(e) =
                    crate::db::insert_metronome_lap(pool, *event_id, &uname, delta_ms).await
            {
                tracing::warn!("Failed to persist metronome lap: {e}");
            }

            let prev_best = state
                .read()
                .leaderboard
                .iter()
                .find(|e| e.0 == uname)
                .map(|e| e.2);
            let is_pb = match prev_best {
                Some(prev) => delta_ms < prev,
                None => true,
            };

            state
                .write()
                .update_leaderboard(&uname, &player.pname, delta_ms);

            let tier_str = tier_label(delta)
                .map(|t| format!(" [{t}]"))
                .unwrap_or_default();

            if is_pb {
                sender.packets(mtc(
                    format!("New best! Off by: {:.3}s{}", delta.as_secs_f64(), tier_str)
                        .light_green(),
                    Some(player.ucid),
                ))?;
            } else {
                sender.packets(mtc(
                    format!("Off by: {:.3}s{}", delta.as_secs_f64(), tier_str).yellow(),
                    Some(player.ucid),
                ))?;
            }

            let best_in_lb = state
                .read()
                .leaderboard
                .iter()
                .find(|e| e.0 == uname)
                .map(|e| e.2);

            let _ = ui
                .assign_player(
                    player.ucid,
                    MetronomeConnectionProps {
                        uname: uname.clone(),
                        in_run: false,
                        best_delta_ms: best_in_lb,
                    },
                )
                .await;
        }
    }

    refresh_ui(&state, &ui);
    Ok(())
}

fn tier_label(delta: Duration) -> Option<&'static str> {
    let ms = delta.as_millis();
    if ms <= 100 {
        Some("Platinum")
    } else if ms <= 500 {
        Some("Gold")
    } else if ms <= 2000 {
        Some("Silver")
    } else if ms <= 5000 {
        Some("Bronze")
    } else {
        None
    }
}
