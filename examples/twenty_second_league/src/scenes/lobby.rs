use std::time::Duration;

use kitcar::{combos::Combo, time::countdown::Countdown};

use crate::{
    combo::ComboExt, components::{RootPhase, RootProps}, GameState
};

pub async fn lobby(
    insim: insim::builder::SpawnedHandle,
    combo: Combo<ComboExt>,
    ui: crate::MyUi,
    lobby_duration: Duration,
) -> anyhow::Result<GameState> {
    let mut packets = insim.subscribe();

    let mut countdown = Countdown::new(
        Duration::from_secs(1),
        lobby_duration.as_secs() as u32, // FIXME
    );

    loop {
        tokio::select! {
            remaining = countdown.tick() => match remaining {
                Some(_) => {
                    println!("Waiting for lobby to complete!");
                    let remaining_duration = countdown.remaining_duration();

                    let _ = ui.update(RootProps {
                        show: true,
                        phase: RootPhase::Lobby {
                            remaining: remaining_duration
                        }
                    });
                },
                None => {
                    break;
                }
            },
            packet = packets.recv() => match packet {
                Ok(packet) => {
                    println!("PhaseLobby: {:?}", packet);
                },
                _ => {}
            }
        }
    }

    Ok(GameState::Round { round: 1, combo })
}
