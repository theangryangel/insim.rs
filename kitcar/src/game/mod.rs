//! Game state

use std::time::Duration;

use insim::{
    core::{track::Track, wind::Wind},
    insim::{RaceInProgress, RaceLaps, StaFlags},
};
use tokio::sync::{mpsc, oneshot};

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

        self.track = Some(sta.track.clone());
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

    /// Spawn a background instance of GameInfo and return a handle so that we can query it
    pub fn spawn(insim: insim::builder::SpawnedHandle, capacity: usize) -> GameHandle {
        let (query_tx, mut query_rx) = mpsc::channel::<oneshot::Sender<GameInfo>>(capacity);

        let _handle = tokio::spawn(async move {
            let mut inner = Self::new();
            let mut packet_rx = insim.subscribe();

            loop {
                tokio::select! {
                    Ok(packet) = packet_rx.recv() => {
                        inner.handle_packet(&packet);
                    }
                    Some(response_tx) = query_rx.recv() => {
                        let _ = response_tx.send(inner.clone());
                    }
                }
            }
        });

        GameHandle { query_tx }
    }
}

#[derive(Debug, Clone)]
/// Handler for Presence
pub struct GameHandle {
    query_tx: mpsc::Sender<oneshot::Sender<GameInfo>>,
}

impl GameHandle {
    /// Request the game state
    pub async fn get(&self) -> GameInfo {
        let (tx, rx) = oneshot::channel();
        self.query_tx.send(tx).await.unwrap(); // FIXME
        rx.await.unwrap() // FIXME
    }
}
