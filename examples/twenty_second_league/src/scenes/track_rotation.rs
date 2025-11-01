use std::time::Duration;

use insim::identifiers::ConnectionId;
use kitcar::{combos::Combo, game::GameHandle};
use tokio::time::timeout;

use crate::{
    combo::ComboExt, GameState
};

pub async fn track_rotation(
    insim: insim::builder::SpawnedHandle,
    combo: Combo<ComboExt>,
    game: GameHandle,
) -> anyhow::Result<GameState> {

    let result = timeout(Duration::from_secs(600), async {
        println!("Transitioning to game");

        let _ = insim.send_command("/end").await;
        let _ = insim.send_message("Waiting for track selection screen...", ConnectionId::ALL);
        game.wait_for_end().await;

        let _ = insim.send_message("Requesting track change", ConnectionId::ALL).await;
        let _ = insim.send_command(&format!("/track {}", combo.track().code())).await;

        let _ = insim.send_message("Waiting for track", ConnectionId::ALL).await;
        game.wait_for_track(*combo.track()).await;

        let _ = insim.send_message("Waiting for game to start", ConnectionId::ALL).await;
        game.wait_for_racing().await;
    }).await;

    if result.is_ok() {
        Ok(GameState::Lobby { combo })
    } else {
        insim.send_message("Timed out waiting for track change and player ready. Back to idle!", ConnectionId::ALL).await?;
        Ok(GameState::Idle)
    }
}
