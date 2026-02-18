//! Game state
use std::time::Duration;

use insim::{
    WithRequestId,
    builder::InsimTask,
    core::{track::Track, wind::Wind},
    insim::{RaceInProgress, RaceLaps, StaFlags, TinyType},
};
use tokio::{
    sync::{mpsc, oneshot, watch},
    task::JoinHandle,
};

#[derive(Debug, Default, Clone)]
/// GameInfo
pub struct GameInfo {
    track: Option<Track>,
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
        self.racing = sta.raceinprog.clone();
        self.qualifying_duration = Duration::from_secs(sta.qualmins as u64 * 60);
        self.race_duration = sta.racelaps;

        self.track = Some(sta.track);
        self.weather = Some(sta.weather);
        self.wind = Some(sta.wind);

        self.flags = sta.flags;
    }

    /// Handle packet updates
    pub fn handle_packet(&mut self, packet: &insim::Packet) {
        #[allow(clippy::single_match)] // we'll come back through to add more support for other
        // stuff later
        match packet {
            insim::Packet::Sta(sta) => self.sta(sta),
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

    /// Lost game watch channel
    #[error("Lost game watch channel")]
    WatchChannelClosed,
}

/// Spawn a background instance of GameInfo and return a handle so that we can query it
pub fn spawn(
    insim: insim::builder::InsimTask,
    capacity: usize,
) -> (Game, JoinHandle<Result<(), GameError>>) {
    let (query_tx, mut query_rx) = mpsc::channel(capacity);
    let (tx, rx) = watch::channel(GameInfo::new());

    let handle = tokio::spawn(async move {
        let result: Result<(), GameError> = async {
            let mut packet_rx = insim.subscribe();
            let mut query_rx_closed = false;

            // Make the relevant background requests that we *must* have. If the user doesnt use
            // spawn it's upto them to handle this.
            insim.send(TinyType::Sst.with_request_id(1)).await?;

            loop {
                tokio::select! {
                    packet = packet_rx.recv() => {
                        match packet {
                            Ok(packet) => {
                                tx.send_modify(|inner| inner.handle_packet(&packet));
                            }
                            Err(_) => return Err(GameError::InsimHandleLost),
                        }
                    }
                    query = query_rx.recv(), if !query_rx_closed => {
                        match query {
                            Some(GameQuery::Get { response_tx }) => {
                                let _ = response_tx.send(tx.borrow().clone());
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

    (
        Game {
            query_tx,
            watch: rx,
        },
        handle,
    )
}

#[derive(Debug)]
enum GameQuery {
    Get {
        response_tx: oneshot::Sender<GameInfo>,
    },
}

#[derive(Debug, Clone)]
/// Handler for Presence
pub struct Game {
    query_tx: mpsc::Sender<GameQuery>,
    watch: watch::Receiver<GameInfo>,
}

impl Game {
    /// Request the game state
    pub async fn get(&self) -> Result<GameInfo, GameError> {
        let (response_tx, rx) = oneshot::channel();
        self.query_tx
            .send(GameQuery::Get { response_tx })
            .await
            .map_err(|_| GameError::QueryChannelClosed)?;
        rx.await.map_err(|_| GameError::ResponseChannelClosed)
    }

    /// Wait for a given state
    pub async fn wait_for<F: Fn(&GameInfo) -> bool>(
        &mut self,
        predicate: F,
    ) -> Result<(), GameError> {
        let _ = self
            .watch
            .wait_for(predicate)
            .await
            .map_err(|_| GameError::WatchChannelClosed)?;
        Ok(())
    }

    /// Wait for the end
    pub async fn wait_for_end(&mut self) -> Result<(), GameError> {
        self.wait_for(|info| !info.flags.is_in_game() && matches!(info.racing, RaceInProgress::No))
            .await
    }

    /// Wait for track to load
    pub async fn wait_for_track(&mut self, track: Track) -> Result<(), GameError> {
        self.wait_for(|info| {
            tracing::debug!("waiting for track {:?}", info);
            if let Some(state_track) = info.track.as_ref()
                && state_track == &track
                && !info.flags.is_in_game()
                && matches!(info.racing, RaceInProgress::No)
            {
                true
            } else {
                false
            }
        })
        .await
    }

    /// Wait for track to load
    pub async fn wait_for_racing(&mut self) -> Result<(), GameError> {
        self.wait_for(|info| {
            tracing::debug!("waiting for racing {:?}", info);
            info.flags.is_in_game() && matches!(info.racing, RaceInProgress::Racing)
        })
        .await
    }

    /// Request track rotation
    pub async fn track_rotation(
        &mut self,
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
        }

        tracing::info!("Waiting for all players to hit ready");
        self.wait_for_racing().await?;
        Ok(())
    }
}
