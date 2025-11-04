use std::time::Duration;

use insim::identifiers::ConnectionId;
use kitcar::combos::Combo;
use tokio::time::timeout;

use crate::{
    Context, GameState,
    combo::ComboExt,
    components::{RootProps, RootScene},
};

pub async fn track_rotation(
    cx: Context,
    combo: Combo<ComboExt>,
    game_id: i64,
) -> anyhow::Result<Option<GameState>> {
    let _ = cx.ui.update(RootProps {
        scene: RootScene::TrackRotation {
            combo: combo.clone(),
        },
    });

    tokio::select! {
        // It's ok for this timeout to be in tokio::select! because our only other arm right now is the
        // cx cancellationtoken, so if we're shutting down we're ok with this cancelling.
        result = timeout(Duration::from_secs(300), async {
            tracing::info!("Changing track and layout combo");

            let _ = cx.insim.send_command("/end").await;
            let _ = cx
                .insim
                .send_message("Waiting for track selection screen...", ConnectionId::ALL);
            cx.game.wait_for_end().await;

            let _ = cx
                .insim
                .send_message("Requesting track change", ConnectionId::ALL)
                .await;

            let _ = cx
                .insim
                .send_command(&format!("/track {}", combo.track().code()))
                .await;

            if let Some(laps) = combo.extensions().laps.as_ref() {
                let _ = cx.insim.send_command(&format!("/laps {}", laps)).await;
            }

            let _ = cx
                .insim
                .send_message("Waiting for track", ConnectionId::ALL)
                .await;
            cx.game.wait_for_track(*combo.track()).await;

            // FIXME: how do we check that the layout loaded safely
            if let Some(lyt) = combo.layout().as_ref() {
                let _ = cx.insim.send_command(&format!("/axload {}", lyt)).await;
            }

            let _ = cx
                .insim
                .send_message("Waiting for game to start", ConnectionId::ALL)
                .await;
            cx.game.wait_for_racing().await;
        }) => {
            if result.is_ok() {
                Ok(Some(GameState::Lobby { combo, game_id }))
            } else {
                cx.insim
                    .send_message(
                        "Timed out waiting for track change and player ready. Back to idle!",
                        ConnectionId::ALL,
                    )
                    .await?;
                Ok(Some(GameState::Idle))
            }
        },
        _ = cx.shutdown.cancelled() => {
            Ok(None)
        }
    }
}
