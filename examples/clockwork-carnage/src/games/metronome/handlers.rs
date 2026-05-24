use std::time::Duration;

use insim::{
    Colour,
    core::object::{
        ObjectInfo,
        insim::{InsimCheckpoint, InsimCheckpointKind},
    },
    identifiers::ConnectionId,
    insim::{Npl, RaceLaps, Tiny, TinyType, Toc, Uco},
};
use kitcar::{
    AppError, Connected, Disconnected, Event, Game, Packet, Presence, RaceEnded, Sender, State,
    track_rotation, util::mtc,
};

use super::{
    config::MIN_PLAYERS,
    events::{SetupAborted, SetupComplete},
    state::{Metronome, MetronomePhase, PlayerInfo},
    ui::{MetronomeConnectionProps, MetronomeUi},
};

fn start_setup(state: &State<Metronome>, game: &Game, sender: &Sender, ui: &MetronomeUi) {
    let (track, layout, setup_timeout, setup_cancel) = {
        let mut m = state.write();
        m.phase = MetronomePhase::SettingUp;
        let token = m.make_setup_cancel();
        (
            m.config.track,
            m.config.layout.clone(),
            m.config.setup_timeout,
            token,
        )
    };

    let _ = sender.packets(mtc(
        "Metronome - setting up track, hit /ready when prompted.",
        Some(ConnectionId::ALL),
    ));
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

fn refresh_ui(state: &State<Metronome>, ui: &MetronomeUi) {
    let snapshot = state.read().snapshot();
    ui.assign_global(snapshot);
}

pub(super) async fn on_connected(
    Event(Connected(info)): Event<Connected>,
    state: State<Metronome>,
    presence: Presence,
    ui: MetronomeUi,
    game: Game,
    sender: Sender,
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
    let should_start = {
        let m = state.read();
        m.phase == MetronomePhase::Waiting && presence.count() >= MIN_PLAYERS
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
    state: State<Metronome>,
    presence: Presence,
    sender: Sender,
    ui: MetronomeUi,
) -> Result<(), AppError> {
    if presence.count() >= MIN_PLAYERS {
        refresh_ui(&state, &ui);
        return Ok(());
    }
    let phase = state.read().phase;
    match phase {
        MetronomePhase::Waiting => {
            refresh_ui(&state, &ui);
        },
        MetronomePhase::SettingUp => {
            state.write().cancel_setup();
            refresh_ui(&state, &ui);
        },
        MetronomePhase::Racing => {
            {
                let mut m = state.write();
                m.players.clear();
                m.active_runs.clear();
                m.phase = MetronomePhase::Waiting;
            }
            sender.packets(mtc(
                "Metronome - not enough players, restarting.",
                Some(ConnectionId::ALL),
            ))?;
            refresh_ui(&state, &ui);
        },
    }
    Ok(())
}

pub(super) async fn on_setup_complete(
    _: Event<SetupComplete>,
    state: State<Metronome>,
    sender: Sender,
    ui: MetronomeUi,
) -> Result<(), AppError> {
    let target_secs = {
        let mut m = state.write();
        if m.phase != MetronomePhase::SettingUp {
            return Ok(());
        }
        m.phase = MetronomePhase::Racing;
        m.clear_setup_cancel();
        m.config.target.as_secs_f64()
    };
    sender.packets(mtc(
        format!("Metronome - target: {target_secs:.1}s. Cross checkpoint 1 to start."),
        Some(ConnectionId::ALL),
    ))?;
    sender.packet(insim::Packet::Tiny(Tiny {
        subt: TinyType::Npl,
        ..Default::default()
    }))?;
    refresh_ui(&state, &ui);
    Ok(())
}

pub(super) async fn on_setup_aborted(
    _: Event<SetupAborted>,
    state: State<Metronome>,
    presence: Presence,
    ui: MetronomeUi,
    game: Game,
    sender: Sender,
) -> Result<(), AppError> {
    {
        let mut m = state.write();
        if m.phase != MetronomePhase::SettingUp {
            return Ok(());
        }
        m.phase = MetronomePhase::Waiting;
        m.clear_setup_cancel();
    }
    sender.packets(mtc(
        "Metronome - setup failed, restarting.",
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
    state: State<Metronome>,
    presence: Presence,
    ui: MetronomeUi,
    game: Game,
    sender: Sender,
) -> Result<(), AppError> {
    {
        let mut m = state.write();
        if m.phase != MetronomePhase::Racing {
            return Ok(());
        }
        m.players.clear();
        m.active_runs.clear();
        m.phase = MetronomePhase::Waiting;
    }
    refresh_ui(&state, &ui);
    if presence.count() >= MIN_PLAYERS {
        start_setup(&state, &game, &sender, &ui);
    }
    Ok(())
}

pub(super) async fn on_npl(
    Packet(npl): Packet<Npl>,
    state: State<Metronome>,
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

pub(super) async fn on_toc(
    Packet(toc): Packet<Toc>,
    state: State<Metronome>,
) -> Result<(), AppError> {
    let mut m = state.write();
    if let Some(p) = m.players.get_mut(&toc.plid) {
        p.ucid = toc.newucid;
    }
    if let Some(r) = m.active_runs.get_mut(&toc.plid) {
        r.0 = toc.newucid;
    }
    Ok(())
}

pub(super) async fn on_uco(
    Packet(uco): Packet<Uco>,
    state: State<Metronome>,
    sender: Sender,
    ui: MetronomeUi,
    presence: Presence,
) -> Result<(), AppError> {
    let ObjectInfo::InsimCheckpoint(InsimCheckpoint { kind, .. }) = uco.info else {
        return Ok(());
    };
    let is_cp1 = matches!(kind, InsimCheckpointKind::Checkpoint1);
    let is_finish = matches!(kind, InsimCheckpointKind::Finish);
    if !is_cp1 && !is_finish {
        return Ok(());
    }

    let player = {
        let m = state.read();
        m.players.get(&uco.plid).cloned()
    };
    let Some(player) = player else {
        return Ok(());
    };
    if player.ptype.contains(insim::insim::PlayerType::AI) {
        return Ok(());
    }

    if is_cp1 {
        let start_time = uco.time;
        let _ = state
            .write()
            .active_runs
            .insert(uco.plid, (player.ucid, player.uname.clone(), start_time));
        sender.packets(mtc(
            "Run started - reach the finish!".light_green(),
            Some(player.ucid),
        ))?;
        let _ = ui
            .assign_player(
                player.ucid,
                MetronomeConnectionProps {
                    uname: player.uname.clone(),
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

            if let Some(pkt) = presence.spec(player.ucid) {
                let _ = sender.packet(pkt);
            }

            let db = state.read().db.clone();
            if let Some((pool, event_id)) = &db {
                if let Err(e) =
                    crate::db::insert_metronome_lap(pool, *event_id, &uname, delta_ms).await
                {
                    tracing::warn!("Failed to persist metronome lap: {e}");
                }
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
