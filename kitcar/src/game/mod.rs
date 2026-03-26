//! Game state
use std::time::Duration;

use insim::{
    WithRequestId,
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
        if let insim::Packet::Sta(sta) = packet {
            self.sta(sta)
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

#[derive(Debug)]
enum GameMessage {
    Get {
        response_tx: oneshot::Sender<GameInfo>,
    },
    End {
        response_tx: oneshot::Sender<Result<(), GameError>>,
    },
    ChangeTrack {
        track: Track,
        response_tx: oneshot::Sender<Result<(), GameError>>,
    },
    ChangeLaps {
        laps: RaceLaps,
        response_tx: oneshot::Sender<Result<(), GameError>>,
    },
    ChangeWind {
        wind: u8,
        response_tx: oneshot::Sender<Result<(), GameError>>,
    },
    AxClear {
        response_tx: oneshot::Sender<Result<(), GameError>>,
    },
    AxLoad {
        layout: String,
        response_tx: oneshot::Sender<Result<(), GameError>>,
    },
    Restart {
        response_tx: oneshot::Sender<Result<(), GameError>>,
    },
    Qualify {
        response_tx: oneshot::Sender<Result<(), GameError>>,
    },
    Reinit {
        response_tx: oneshot::Sender<Result<(), GameError>>,
    },
    ChangeWeather {
        weather: u8,
        response_tx: oneshot::Sender<Result<(), GameError>>,
    },
    ChangeQual {
        minutes: u8,
        response_tx: oneshot::Sender<Result<(), GameError>>,
    },
    PitAll {
        response_tx: oneshot::Sender<Result<(), GameError>>,
    },
}

/// Spawn a background instance of GameInfo and return a handle so that we can query it
pub fn spawn(
    insim: insim::builder::InsimTask,
    capacity: usize,
) -> (Game, JoinHandle<Result<(), GameError>>) {
    let (tx, mut rx) = mpsc::channel(capacity);

    let handle = tokio::spawn(async move {
        let result: Result<(), GameError> = async {
            let mut packet_rx = insim.subscribe();
            let mut inner = GameInfo::new();
            let mut rx_closed = false;

            // Make the relevant background requests that we *must* have. If the user doesnt use
            // spawn it's upto them to handle this.
            insim.send(TinyType::Sst.with_request_id(1)).await?;

            loop {
                tokio::select! {
                    packet = packet_rx.recv() => {
                        match packet {
                            Ok(packet) => inner.handle_packet(&packet),
                            Err(_) => return Err(GameError::InsimHandleLost),
                        }
                    }
                    msg = rx.recv(), if !rx_closed => {
                        match msg {
                            Some(GameMessage::Get { response_tx }) => {
                                let _ = response_tx.send(inner.clone());
                            }
                            Some(GameMessage::End { response_tx }) => {
                                let res = insim.send_command("/end").await.map_err(GameError::from);
                                let _ = response_tx.send(res);
                            }
                            Some(GameMessage::ChangeTrack { track, response_tx }) => {
                                let res = insim
                                    .send_command(format!("/track {track}"))
                                    .await
                                    .map_err(GameError::from);
                                let _ = response_tx.send(res);
                            }
                            Some(GameMessage::ChangeLaps { laps, response_tx }) => {

                                let cmd = match laps {
                                    RaceLaps::Untimed => "/laps no".to_string(),
                                    RaceLaps::Hours(h) => format!("/hours {h}"),
                                    o => format!("/laps {}", Into::<u8>::into(o)),
                                };

                                let res = insim
                                    .send_command(cmd)
                                    .await
                                    .map_err(GameError::from);
                                let _ = response_tx.send(res);
                            }
                            Some(GameMessage::ChangeWind { wind, response_tx }) => {
                                let res = insim
                                    .send_command(format!("/wind {wind}"))
                                    .await
                                    .map_err(GameError::from);
                                let _ = response_tx.send(res);
                            }
                            Some(GameMessage::AxClear { response_tx }) => {
                                let res = insim
                                    .send_command("/axclear")
                                    .await
                                    .map_err(GameError::from);
                                let _ = response_tx.send(res);
                            }
                            Some(GameMessage::AxLoad { layout, response_tx }) => {
                                let res = insim
                                    .send_command(format!("/axload {layout}"))
                                    .await
                                    .map_err(GameError::from);
                                let _ = response_tx.send(res);
                            }
                            Some(GameMessage::Restart { response_tx }) => {
                                let res = insim.send_command("/restart").await.map_err(GameError::from);
                                let _ = response_tx.send(res);
                            }
                            Some(GameMessage::Qualify { response_tx }) => {
                                let res = insim.send_command("/qualify").await.map_err(GameError::from);
                                let _ = response_tx.send(res);
                            }
                            Some(GameMessage::Reinit { response_tx }) => {
                                let res = insim.send_command("/reinit").await.map_err(GameError::from);
                                let _ = response_tx.send(res);
                            }
                            Some(GameMessage::ChangeWeather { weather, response_tx }) => {
                                let res = insim
                                    .send_command(format!("/weather {weather}"))
                                    .await
                                    .map_err(GameError::from);
                                let _ = response_tx.send(res);
                            }
                            Some(GameMessage::ChangeQual { minutes, response_tx }) => {
                                let res = insim
                                    .send_command(format!("/qual {minutes}"))
                                    .await
                                    .map_err(GameError::from);
                                let _ = response_tx.send(res);
                            }
                            Some(GameMessage::PitAll { response_tx }) => {
                                let res = insim
                                    .send_command("/pit_all")
                                    .await
                                    .map_err(GameError::from);
                                let _ = response_tx.send(res);
                            }
                            None => {
                                rx_closed = true;
                            }
                        }
                    }
                }
            }
        }
        .await;
        result
    });

    (Game { tx }, handle)
}

#[derive(Debug, Clone)]
/// Handler for game state
pub struct Game {
    tx: mpsc::Sender<GameMessage>,
}

impl Game {
    async fn send_command(
        &self,
        msg: GameMessage,
        rx: oneshot::Receiver<Result<(), GameError>>,
    ) -> Result<(), GameError> {
        self.tx
            .send(msg)
            .await
            .map_err(|_| GameError::QueryChannelClosed)?;
        rx.await.map_err(|_| GameError::ResponseChannelClosed)?
    }

    /// Request the current game state.
    pub async fn get(&self) -> Result<GameInfo, GameError> {
        let (response_tx, rx) = oneshot::channel();
        self.tx
            .send(GameMessage::Get { response_tx })
            .await
            .map_err(|_| GameError::QueryChannelClosed)?;
        rx.await.map_err(|_| GameError::ResponseChannelClosed)
    }

    /// Send the `/end` command.
    pub async fn end(&self) -> Result<(), GameError> {
        let (response_tx, rx) = oneshot::channel();
        self.send_command(GameMessage::End { response_tx }, rx)
            .await
    }

    /// Send a track change command.
    pub async fn change_track(&self, track: Track) -> Result<(), GameError> {
        let (response_tx, rx) = oneshot::channel();
        self.send_command(GameMessage::ChangeTrack { track, response_tx }, rx)
            .await
    }

    /// Send a laps change command.
    pub async fn change_laps(&self, laps: RaceLaps) -> Result<(), GameError> {
        let (response_tx, rx) = oneshot::channel();
        self.send_command(GameMessage::ChangeLaps { laps, response_tx }, rx)
            .await
    }

    /// Send a wind change command.
    pub async fn change_wind(&self, wind: u8) -> Result<(), GameError> {
        let (response_tx, rx) = oneshot::channel();
        self.send_command(GameMessage::ChangeWind { wind, response_tx }, rx)
            .await
    }

    /// Clear the autocross layout.
    pub async fn ax_clear(&self) -> Result<(), GameError> {
        let (response_tx, rx) = oneshot::channel();
        self.send_command(GameMessage::AxClear { response_tx }, rx)
            .await
    }

    /// Start a race.
    pub async fn restart(&self) -> Result<(), GameError> {
        let (response_tx, rx) = oneshot::channel();
        self.send_command(GameMessage::Restart { response_tx }, rx)
            .await
    }

    /// Start qualifying.
    pub async fn qualify(&self) -> Result<(), GameError> {
        let (response_tx, rx) = oneshot::channel();
        self.send_command(GameMessage::Qualify { response_tx }, rx)
            .await
    }

    /// Total restart — removes all connections.
    pub async fn reinit(&self) -> Result<(), GameError> {
        let (response_tx, rx) = oneshot::channel();
        self.send_command(GameMessage::Reinit { response_tx }, rx)
            .await
    }

    /// Set weather/lighting.
    pub async fn change_weather(&self, weather: u8) -> Result<(), GameError> {
        let (response_tx, rx) = oneshot::channel();
        self.send_command(
            GameMessage::ChangeWeather {
                weather,
                response_tx,
            },
            rx,
        )
        .await
    }

    /// Set qualifying duration in minutes. 0 = no qualifying.
    pub async fn change_qual(&self, minutes: u8) -> Result<(), GameError> {
        let (response_tx, rx) = oneshot::channel();
        self.send_command(
            GameMessage::ChangeQual {
                minutes,
                response_tx,
            },
            rx,
        )
        .await
    }

    /// Pit all
    pub async fn pit_all(&self) -> Result<(), GameError> {
        let (response_tx, rx) = oneshot::channel();
        self.send_command(GameMessage::PitAll { response_tx }, rx)
            .await
    }

    /// Load an autocross layout.
    pub async fn ax_load(&self, layout: impl Into<String>) -> Result<(), GameError> {
        let (response_tx, rx) = oneshot::channel();
        self.send_command(
            GameMessage::AxLoad {
                layout: layout.into(),
                response_tx,
            },
            rx,
        )
        .await
    }

    /// Poll until the predicate returns `true`.
    ///
    /// `poll_interval` controls how frequently the predicate is evaluated.
    pub async fn wait_for<F: Fn(&GameInfo) -> bool>(
        &self,
        predicate: F,
        poll_interval: std::time::Duration,
    ) -> Result<(), GameError> {
        let mut interval = time::interval(poll_interval);
        loop {
            let _ = interval.tick().await;
            if predicate(&self.get().await?) {
                return Ok(());
            }
        }
    }

    /// Wait until the game is no longer in progress.
    pub async fn wait_for_end(&self) -> Result<(), GameError> {
        self.wait_for(
            |info| !info.flags.is_in_game() && matches!(info.racing, RaceInProgress::No),
            std::time::Duration::from_millis(500),
        )
        .await
    }

    /// Wait until the given track is loaded and the server is at the selection screen.
    pub async fn wait_for_track(&self, track: Track) -> Result<(), GameError> {
        self.wait_for(
            |info| {
                tracing::debug!("waiting for track {:?}", info);
                info.track.as_ref() == Some(&track)
                    && !info.flags.is_in_game()
                    && matches!(info.racing, RaceInProgress::No)
            },
            std::time::Duration::from_millis(500),
        )
        .await
    }

    /// Wait until racing is in progress.
    pub async fn wait_for_racing(&self) -> Result<(), GameError> {
        self.wait_for(
            |info| {
                tracing::debug!("waiting for racing {:?}", info);
                info.flags.is_in_game() && matches!(info.racing, RaceInProgress::Racing)
            },
            std::time::Duration::from_millis(500),
        )
        .await
    }

    /// Request track rotation
    pub async fn track_rotation(
        &self,
        track: Track,
        laps: RaceLaps,
        wind: u8,
        layout: Option<String>,
    ) -> Result<(), GameError> {
        let current = self.get().await?;

        if current.track != Some(track) {
            tracing::info!("/end");
            self.end().await?;
            tracing::info!("waiting for track selection screen");
            self.wait_for_end().await?;

            tracing::info!("Requesting track change");
            self.change_track(track).await?;
        }

        tracing::info!("Requesting laps change");
        self.change_laps(laps).await?;

        tracing::info!("Requesting wind change");
        self.change_wind(wind).await?;

        self.ax_clear().await?;

        if let Some(layout) = layout {
            tracing::info!("Requesting layout load: {}", layout);
            self.ax_load(&layout).await?;
        }

        tracing::info!("Waiting for all players to hit ready");
        self.wait_for_racing().await?;
        Ok(())
    }
}
