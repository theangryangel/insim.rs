//! Track rotation

use insim::{builder::SpawnedHandle, core::track::Track, insim::RaceLaps};
use tokio::task::{JoinError, JoinHandle};

use crate::game::GameHandle;
#[derive(Debug)]
/// Request a track rotation
pub struct TrackRotation {
    handle: JoinHandle<insim::Result<()>>,
}

impl TrackRotation {
    /// Request a track rotation
    pub fn request(
        game: GameHandle,
        insim: SpawnedHandle,

        track: Track,
        layout: Option<String>,
        laps: RaceLaps,
        wind: Option<u8>,
    ) -> Self {
        let handle = tokio::spawn(async move {
            tracing::info!("/end");
            insim.send_command("/end").await?;
            tracing::info!("waiting for track selection screen");
            game.wait_for_end().await;

            tracing::info!("Requesting track change");
            insim.send_command(&format!("/track {}", &track)).await?;

            let laps: u8 = laps.into();

            tracing::info!("Requesting laps change");
            insim.send_command(&format!("/laps {:?}", laps)).await?;

            tracing::info!("Requesting wind change");
            insim.send_command(&format!("/wind {:?}", &wind)).await?;

            tracing::info!("Requesting layout load");
            insim.send_command(&format!("/axload {:?}", &layout)).await?;

            tracing::info!("Waiting for all players to hit ready");
            game.wait_for_racing().await;

            Ok(())
        });

        Self {
            handle
        }
    }

    /// Wait for completion
    pub async fn poll(&mut self) -> Result<insim::Result<()>, JoinError> {
        (&mut self.handle).await
    }

    /// Abort a track rotation
    pub fn abort(&mut self) {
        self.handle.abort();
    }
}

impl Drop for TrackRotation {
    fn drop(&mut self) {
        self.abort();
    }
}
