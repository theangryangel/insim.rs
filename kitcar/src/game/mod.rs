//! Game state
use std::time::Duration;

use insim::{
    WithRequestId,
    builder::InsimTask,
    core::{track::Track, wind::Wind},
    insim::{RaceInProgress, RaceLaps, StaFlags, TinyType},
};
use tokio::{
    sync::{mpsc, oneshot},
    task::JoinHandle,
    time,
};

#[derive(Debug, Default, Clone)]
/// GameInfo
pub struct GameInfo {
    track: Option<Track>,
    layout: Option<String>,
    weather: Option<u8>,
    wind: Option<Wind>,
    racing: RaceInProgress,
    qualifying_duration: Duration,
    race_duration: RaceLaps,
    flags: StaFlags,
}

impl GameInfo {
    /// New!
    pub fn new() -> Self {
        Self::default()
    }

    /// Current track
    pub fn current_track(&self) -> Option<&Track> {
        self.track.as_ref()
    }

    /// Currently loaded layout name, or `None` if no layout is loaded.
    pub fn current_layout(&self) -> Option<&str> {
        self.layout.as_deref()
    }

    /// Fetch the weather
    pub fn weather(&self) -> Option<&u8> {
        self.weather.as_ref()
    }

    /// Fetch the wind
    pub fn wind(&self) -> Option<&Wind> {
        self.wind.as_ref()
    }

    /// Fetch RaceInProgress
    pub fn racing(&self) -> &RaceInProgress {
        &self.racing
    }

    /// Fetch qualifying_duration
    pub fn qualifying_duration(&self) -> &Duration {
        &self.qualifying_duration
    }

    /// Fetch race_duration
    pub fn race_duration(&self) -> &RaceLaps {
        &self.race_duration
    }

    /// Fetch the game state flags
    pub fn flags(&self) -> &StaFlags {
        &self.flags
    }

    fn sta(&mut self, sta: &insim::insim::Sta) {
        // Track change clears the layout — clear defensively before Axi arrives.
        if self.track != Some(sta.track) {
            self.layout = None;
        }

        self.racing = sta.raceinprog.clone();
        self.qualifying_duration = Duration::from_secs(sta.qualmins as u64 * 60);
        self.race_duration = sta.racelaps;

        self.track = Some(sta.track);
        self.weather = Some(sta.weather);
        self.wind = Some(sta.wind);

        self.flags = sta.flags;
    }

    fn axi(&mut self, axi: &insim::insim::Axi) {
        self.layout = if axi.lname.is_empty() {
            None
        } else {
            Some(axi.lname.clone())
        };
    }

    /// Handle packet updates
    pub fn handle_packet(&mut self, packet: &insim::Packet) {
        match packet {
            insim::Packet::Sta(sta) => self.sta(sta),
            insim::Packet::Axi(axi) => self.axi(axi),
            _ => {},
        }
    }
}

#[derive(Debug, thiserror::Error)]
/// GameError
pub enum GameError {
    /// Insim error
    #[error("Insim: {0}")]
    Insim(#[from] insim::Error),

    /// Lost Insim packet stream
    #[error("Lost Insim packet stream")]
    InsimHandleLost,

    /// Lost game query channel
    #[error("Lost game query channel")]
    QueryChannelClosed,

    /// Lost game response channel
    #[error("Lost game response channel")]
    ResponseChannelClosed,

}

/// Spawn a background instance of GameInfo and return a handle so that we can query it
pub fn spawn(
    insim: insim::builder::InsimTask,
    capacity: usize,
) -> (Game, JoinHandle<Result<(), GameError>>) {
    let (query_tx, mut query_rx) = mpsc::channel(capacity);

    let handle = tokio::spawn(async move {
        let result: Result<(), GameError> = async {
            let mut packet_rx = insim.subscribe();
            let mut inner = GameInfo::new();
            let mut query_rx_closed = false;

            // Make the relevant background requests that we *must* have. If the user doesnt use
            // spawn it's upto them to handle this.
            insim.send(TinyType::Sst.with_request_id(1)).await?;
            insim.send(TinyType::Axi.with_request_id(1)).await?;

            loop {
                tokio::select! {
                    packet = packet_rx.recv() => {
                        match packet {
                            Ok(packet) => inner.handle_packet(&packet),
                            Err(_) => return Err(GameError::InsimHandleLost),
                        }
                    }
                    query = query_rx.recv(), if !query_rx_closed => {
                        match query {
                            Some(GameQuery::Get { response_tx }) => {
                                let _ = response_tx.send(inner.clone());
                            }
                            None => {
                                query_rx_closed = true;
                            }
                        }
                    }
                }
            }
        }
        .await;
        result
    });

    (Game { query_tx }, handle)
}

#[derive(Debug)]
enum GameQuery {
    Get {
        response_tx: oneshot::Sender<GameInfo>,
    },
}

#[derive(Debug, Clone)]
/// Handler for game state
pub struct Game {
    query_tx: mpsc::Sender<GameQuery>,
}

impl Game {
    /// Request the current game state.
    pub async fn get(&self) -> Result<GameInfo, GameError> {
        let (response_tx, rx) = oneshot::channel();
        self.query_tx
            .send(GameQuery::Get { response_tx })
            .await
            .map_err(|_| GameError::QueryChannelClosed)?;
        rx.await.map_err(|_| GameError::ResponseChannelClosed)
    }

    /// Poll until the predicate returns `true`.
    pub async fn wait_for<F: Fn(&GameInfo) -> bool>(
        &self,
        predicate: F,
    ) -> Result<(), GameError> {
        let mut interval = time::interval(std::time::Duration::from_millis(500));
        loop {
            let _ = interval.tick().await;
            if predicate(&self.get().await?) {
                return Ok(());
            }
        }
    }

    /// Wait until the game is no longer in progress.
    pub async fn wait_for_end(&self) -> Result<(), GameError> {
        self.wait_for(|info| !info.flags.is_in_game() && matches!(info.racing, RaceInProgress::No))
            .await
    }

    /// Wait until the named layout is confirmed loaded.
    pub async fn wait_for_layout(&self, layout: &str) -> Result<(), GameError> {
        self.wait_for(|info| info.layout.as_deref() == Some(layout))
            .await
    }

    /// Wait until the given track is loaded and the server is at the selection screen.
    pub async fn wait_for_track(&self, track: Track) -> Result<(), GameError> {
        self.wait_for(|info| {
            tracing::debug!("waiting for track {:?}", info);
            info.track.as_ref() == Some(&track)
                && !info.flags.is_in_game()
                && matches!(info.racing, RaceInProgress::No)
        })
        .await
    }

    /// Wait until racing is in progress.
    pub async fn wait_for_racing(&self) -> Result<(), GameError> {
        self.wait_for(|info| {
            tracing::debug!("waiting for racing {:?}", info);
            info.flags.is_in_game() && matches!(info.racing, RaceInProgress::Racing)
        })
        .await
    }

    /// Request track rotation
    pub async fn track_rotation(
        &self,
        insim: InsimTask,
        track: Track,
        laps: RaceLaps,
        wind: u8,
        layout: Option<String>,
    ) -> Result<(), GameError> {
        let current = self.get().await?;

        if current.track != Some(track) {
            tracing::info!("/end");
            insim.send_command("/end").await?;
            tracing::info!("waiting for track selection screen");
            self.wait_for_end().await?;

            tracing::info!("Requesting track change");
            insim.send_command(format!("/track {track}")).await?;
        }

        let laps: u8 = laps.into();

        tracing::info!("Requesting laps change");
        insim.send_command(format!("/laps {laps}")).await?;

        tracing::info!("Requesting wind change");
        insim.send_command(format!("/wind {wind}")).await?;

        insim.send_command("/axclear").await?;

        if let Some(layout) = layout {
            tracing::info!("Requesting layout load: {}", layout);
            insim.send_command(format!("/axload {layout}")).await?;
            tracing::info!("Waiting for layout to load: {}", layout);
            self.wait_for_layout(&layout).await?;
        }

        tracing::info!("Waiting for all players to hit ready");
        self.wait_for_racing().await?;
        Ok(())
    }
}
