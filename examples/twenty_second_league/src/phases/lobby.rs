use std::time::Duration;

use kitcar::{presence::PresenceHandle, time::countdown::Countdown, ui};

use super::Transition;
use crate::components::{Root, RootPhase, RootProps};

pub(crate) struct PhaseLobby;

impl PhaseLobby {
    pub(crate) fn spawn(
        insim: insim::builder::SpawnedHandle,
        presence: PresenceHandle,
        ui: ui::ManagerHandle<Root>,
    ) -> tokio::task::JoinHandle<Transition> {
        // Using a zero sized struct for this one because we dont actually need it to do anything
        // more than this.

        tokio::spawn(async move {
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

            Transition::Game
        })
    }
}
