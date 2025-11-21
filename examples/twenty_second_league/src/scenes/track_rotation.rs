use std::time::Duration;

use insim::identifiers::ConnectionId;
use kitcar::combos::Combo;
use tokio::time::timeout;

use crate::{
    Context, Scene,
    combo::ComboExt,
    components::{RootProps, RootScene},
    db::game::EventId,
};

#[derive(Debug, Clone)]
pub struct TrackRotation {
    pub combo: Combo<ComboExt>,
    pub game_id: EventId,
}

impl TrackRotation {
    pub async fn run(self, cx: Context) -> anyhow::Result<Option<Scene>> {
        cx.ui.update(RootProps {
            scene: RootScene::TrackRotation {
                combo: self.combo.clone(),
            },
        });

        tokio::select! {
            // It's ok for this timeout to be in tokio::select! because our only other arm right now is the
            // cx cancellationtoken, so if we're shutting down we're ok with this cancelling.
            result = timeout(Duration::from_secs(300), async {
                tracing::info!("Changing track and layout combo");

                cx.insim.send_command("/end").await?;
                cx
                    .insim
                    .send_message("Waiting for track selection screen...", ConnectionId::ALL).await?;
                cx.game.wait_for_end().await;

                cx
                    .insim
                    .send_message("Requesting track change", ConnectionId::ALL)
                    .await?;

                cx
                    .insim
                    .send_command(&format!("/track {}", self.combo.track().code()))
                    .await?;

                // always practise mode and no wind
                cx.insim.send_command("/laps 0").await?;
                cx.insim.send_command("/wind 0").await?;

                let _ = cx
                    .insim
                    .send_message("Waiting for track", ConnectionId::ALL)
                    .await;
                cx.game.wait_for_track(*self.combo.track()).await;

                // FIXME: how do we check that the layout loaded safely
                if let Some(lyt) = self.combo.layout().as_ref() {
                    cx.insim.send_command(&format!("/axload {}", lyt)).await?;
                }

                let _ = cx
                    .insim
                    .send_message("Waiting for game to start", ConnectionId::ALL)
                    .await;
                cx.game.wait_for_racing().await;

                anyhow::Ok(())
            }) => {
                if result.is_ok() {
                    Ok(Some(super::Lobby { combo: self.combo, game_id: self.game_id }.into()))
                } else {
                    cx.insim
                        .send_message(
                            "Timed out waiting for track change and player ready. Back to idle!",
                            ConnectionId::ALL,
                        )
                        .await?;
                    Ok(Some(super::Idle.into()))
                }
            },
            _ = cx.shutdown.cancelled() => {
                Ok(None)
            }
        }
    }
}
