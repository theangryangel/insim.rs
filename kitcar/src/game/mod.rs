//! Game state
use std::time::Duration;

use insim::{
    WithRequestId,
    builder::SpawnedHandle,
    core::{track::Track, wind::Wind},
    insim::{RaceInProgress, RaceLaps, StaFlags, TinyType},
};
use tokio::sync::{mpsc, oneshot, watch};

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

    /// Spawn a background instance of GameInfo and return a handle so that we can query it
    pub fn spawn(insim: insim::builder::SpawnedHandle, capacity: usize) -> GameHandle {
        let (query_tx, mut query_rx) = mpsc::channel(capacity);
        let (tx, rx) = watch::channel(Self::new());

        let _handle = tokio::spawn(async move {
            let mut packet_rx = insim.subscribe();

            // Make the relevant background requests that we *must* have. If the user doesnt use
            // spawn it's upto them to handle this.
            let _ = insim.send(TinyType::Sst.with_request_id(1)).await;

            loop {
                tokio::select! {
                    Ok(packet) = packet_rx.recv() => {
                        tx.send_modify(|inner| inner.handle_packet(&packet) );
                    }
                    Some(query) = query_rx.recv() => {
                        match query {
                            GameQuery::Get { response_tx } => {
                                let _ = response_tx.send(tx.borrow().clone());
                            },
                        }
                    }
                }
            }
        });

        GameHandle {
            query_tx,
            watch: rx,
        }
    }
}

#[derive(Debug)]
enum GameQuery {
    Get {
        response_tx: oneshot::Sender<GameInfo>,
    },
}

#[derive(Debug, Clone)]
/// Handler for Presence
pub struct GameHandle {
    query_tx: mpsc::Sender<GameQuery>,
    watch: watch::Receiver<GameInfo>,
}

impl GameHandle {
    /// Request the game state
    pub async fn get(&self) -> GameInfo {
        let (response_tx, rx) = oneshot::channel();
        self.query_tx
            .send(GameQuery::Get { response_tx })
            .await
            .unwrap(); // FIXME
        rx.await.unwrap() // FIXME
    }

    /// Wait for a given state
    pub async fn wait_for<F: Fn(&GameInfo) -> bool>(&mut self, predicate: F) {
        let _ = self.watch.wait_for(predicate).await.unwrap(); // FIXME
    }

    /// Wait for the end
    pub async fn wait_for_end(&mut self) {
        self.wait_for(|info| {
            if_chain::if_chain! {
                    if !info.flags.is_in_game();
                    if matches!(info.racing, RaceInProgress::No);
                    then {
                        true
                    }
                    else {
                        false
                    }
            }
        })
        .await
    }

    /// Wait for track to load
    pub async fn wait_for_track(&mut self, track: Track) {
        self.wait_for(|info| {
            tracing::debug!("waiting for track {:?}", info);
            if_chain::if_chain! {
                    if let Some(state_track) = info.track.as_ref();
                    if state_track == &track;
                    if !info.flags.is_in_game();
                    if matches!(info.racing, RaceInProgress::No);
                    then {
                        true
                    }
                    else {
                        false
                    }

            }
        })
        .await
    }

    /// Wait for track to load
    pub async fn wait_for_racing(&mut self) {
        self.wait_for(|info| {
            tracing::debug!("waiting for racing {:?}", info);
            if_chain::if_chain! {
                    if info.flags.is_in_game();
                    if matches!(info.racing, RaceInProgress::Racing);
                    then {
                        true
                    }
                    else {
                        false
                    }

            }
        })
        .await
    }

    /// Request track rotation
    pub async fn track_rotation(
        &mut self,
        insim: SpawnedHandle,
        track: Track,
        laps: RaceLaps,
        wind: u8,
        layout: Option<String>,
    ) {
        tracing::info!("/end");
        insim
            .send_command("/end")
            .await
            .expect("FIXME: do not fail");
        tracing::info!("waiting for track selection screen");
        self.wait_for_end().await;

        tracing::info!("Requesting track change");
        insim
            .send_command(&format!("/track {}", &track))
            .await
            .expect("FIXME: do not fail");

        let laps: u8 = laps.into();

        tracing::info!("Requesting laps change");
        insim
            .send_command(&format!("/laps {:?}", laps))
            .await
            .expect("FIXME: do not fail");

        tracing::info!("Requesting wind change");
        insim
            .send_command(&format!("/wind {:?}", &wind))
            .await
            .expect("FIXME: do not fail");

        tracing::info!("Requesting layout load");
        if let Some(layout) = &layout {
            insim
                .send_command(&format!("/axload {:?}", layout))
                .await
                .expect("FIXME: do not fail");
        } else {
            insim
                .send_command("/axclear")
                .await
                .expect("FIXME: do not fail");
        }

        tracing::info!("Waiting for all players to hit ready");
        self.wait_for_racing().await;
    }
}
