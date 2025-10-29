use insim::{Packet, core::track::Track};
use kitcar::runtime::{Result, Transition, context::State};

use crate::MyState;

pub async fn idle(
    insim: insim::builder::SpawnedHandle,
    State(state): State<MyState>,
) -> Result<Transition<MyState>> {
    let mut packets = insim.subscribe();

    loop {
        let packet = packets.recv().await.unwrap();

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
            Packet::Mso(mso) => {
                if_chain::if_chain! {
                    if mso.msg_from_textstart() == "!start";
                    if let Some(conn_info) = state.presence.connection(&mso.ucid).await;
                    if conn_info.admin;
                    then {
                        println!("Transitioning to game");

                        let _ = insim.send_command("/end").await;
                        println!("Waiting for end state");
                        state.game.wait_for_end().await;

                        println!("REquesting track change");
                        let _ = insim.send_command("/track FE1").await;
                        println!("Waiting for track");
                        state.game.wait_for_track(Track::Fe1).await;

                        println!("Waiting for game to start");
                        state.game.wait_for_racing().await;

                        return Ok(Transition::next(super::lobby));
                    }
                }
            },
            _ => {},
        }
    }
}
