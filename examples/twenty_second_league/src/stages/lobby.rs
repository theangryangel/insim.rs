use std::time::Duration;

use kitcar::time::countdown::Countdown;

use crate::{
    GameState, MyContext,
    components::{RootPhase, RootProps},
};

pub async fn lobby(
    insim: insim::builder::SpawnedHandle,
    state: MyContext,
) -> anyhow::Result<GameState> {
    let mut packets = insim.subscribe();

    let mut countdown = Countdown::new(
        Duration::from_secs(1),
        60, // FIXME: pull from config
    );

    loop {
        tokio::select! {
            remaining = countdown.tick() => match remaining {
                Some(_) => {
                    println!("Waiting for lobby to complete!");
                    let remaining_duration = countdown.remaining_duration();

                    let _ = state.ui.update(RootProps {
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

    Ok(GameState::Game)
}
