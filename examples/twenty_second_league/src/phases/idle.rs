use insim::Packet;
use kitcar::presence::PresenceHandle;

use super::Transition;

pub(crate) struct PhaseIdle;

impl PhaseIdle {
    pub(crate) fn spawn(
        insim: insim::builder::SpawnedHandle,
        presence: PresenceHandle,
    ) -> tokio::task::JoinHandle<Transition> {
        // Using a zero sized struct for this one because we dont actually need it to do anything
        // more than this.

        tokio::spawn(async move {
            let mut packets = insim.subscribe();

            loop {
                let packet = packets.recv().await.unwrap();

                println!("Game Idle {packet:?}");

                match packet {
                    Packet::Ncn(ncn) => {
                        insim
                            .send_message(
                                &format!("Welcome. No game is currently in progress."),
                                ncn.ucid,
                            )
                            .await
                            .unwrap();
                    },
                    _ => {},
                }
            }
        })
    }
}
