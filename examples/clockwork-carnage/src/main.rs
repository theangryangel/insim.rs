//! Clockwork carnage. PoC to experiment with the "scene" wrapping idea. go look at
//! twenty_second_league for where this is "going".

use std::{fmt::Debug, time::Duration, usize};

use insim::{core::track::Track, insim::RaceLaps};
use tokio::{sync, task::{JoinError, JoinHandle}, time::{interval, sleep}};

trait HasInsim {
    fn insim(&self) -> insim::builder::SpawnedHandle;
}

// A stage/layer/scene that orchestrates game flow. long live, delegates to other scene in a
// waterfall manner
trait Scene: Send + Sync + 'static {
    type Output: Debug + Send + Sync + 'static;
    type Context: Debug + Send + Sync + Clone;
    fn call(&mut self, ctx: Self::Context) -> impl Future<Output = Self::Output> + Send;
}

struct ActionHandle<T> {
    inner: JoinHandle<T>,
}

impl<T> ActionHandle<T> {
    /// poll
    pub async fn poll(&mut self) -> Result<T, JoinError> {
        (&mut self.inner).await
    }

    /// abort
    pub fn abort(&self) {
        self.inner.abort();
    }
}

impl<T> Drop for ActionHandle<T> {
    fn drop(&mut self) {
        self.inner.abort();
    }
}

// A self-contained multi-step procedure that runs to completion in a cancel safe way
// Useful for things like track rotation
trait Action: Send + Sync + 'static + Sized {
    type Output: Debug + Send + Sync + 'static;
    type Context: Debug + Send + Sync + 'static;

    fn call(self, ctx: Self::Context) -> impl Future<Output=Self::Output> + Send;

    fn spawn(self, ctx: Self::Context) -> ActionHandle<Self::Output> {
        let inner = tokio::spawn(async move { 
            self.call(ctx).await 
        });

        ActionHandle {
            inner
        }
    }
}

#[derive(Debug, Default)]
struct TrackRotation {
    track: Track,
    layout: Option<String>,
    laps: RaceLaps,
    wind: Option<u8>,
}

impl Action for TrackRotation {
    type Output = Result<(), ()>;
    type Context = ();

    async fn call(self, _ctx: Self::Context) -> Self::Output {
        tracing::info!("/end");
        tracing::info!("waiting for track selection screen");
        sleep(Duration::from_secs(1)).await;

        tracing::info!("/track {}", &self.track);
        tracing::info!("Requesting track change");
        sleep(Duration::from_secs(1)).await;

        let laps: u8 = self.laps.into();

        tracing::info!("/laps {:?}", laps);
        tracing::info!("Requesting laps change");

        tracing::info!("/wind {:?}", &self.wind);
        tracing::info!("Requesting wind change");

        tracing::info!("/axload {:?}", &self.layout);
        tracing::info!("Requesting layout load");

        Ok(())
    }
}

struct Supervisor<L: Scene + Clone> {
    inner: L,
    min_players: usize,
    rx: sync::watch::Receiver<usize>,
}

impl<L: Scene + Clone> Supervisor<L> {
    fn supervise(inner: L, min_players: usize, rx: sync::watch::Receiver<usize>) -> Self {
        Self {
            inner, min_players, rx
        }
    }
}

impl<L: Scene + Clone> Scene for Supervisor<L> {
    type Output = L::Output;
    type Context = L::Context;

    async fn call(&mut self, ctx: Self::Context) -> Self::Output {
        loop {
            let min = self.min_players;
            // wait for min_players
            // in the "real world" this would be it's own loop where we also listen for Ncn packets
            // and welcome players
            let _ = self.rx.wait_for(|val| *val > min).await;

            let mut h = tokio::spawn({
                let mut inst = self.inner.clone();
                let ctx = ctx.clone();
                async move { inst.call(ctx).await }
            });

            tokio::select! {
                result = &mut h => {
                    tracing::info!("{:?}", result);
                    match result {
                        Ok(result) => return result,
                        Err(e) => {
                            if e.is_cancelled() {
                                tracing::warn!("Game was cancelled.");
                            } else {
                                tracing::error!("Panicked! {:?}", e);
                            }
                            // If it crashed, we restart the loop
                            continue; 
                        }
                    }
                },
                _ =  self.rx.wait_for(|val| *val < min) => {
                    h.abort();
                    tracing::error!("out of players. going back to the start");
                    // run out of players. go back to the start
                    continue
                }
            }
        }
    }
}

#[derive(Debug, Default, Clone)]
struct ClockworkCarnage;

impl Scene for ClockworkCarnage {
    type Output = anyhow::Result<()>;
    type Context = ();

    async fn call(&mut self, _ctx: Self::Context) -> Self::Output {
        // TODO: handle admin commands

        tracing::info!("Starting...");
        Lobby.call(()).await?;
        Ok(())
    }
}

struct Lobby;
impl Scene for Lobby {
    type Output = anyhow::Result<()>;
    type Context = ();

    async fn call(&mut self, _ctx: Self::Context) -> Self::Output {
        tracing::info!("Lobby started");

        let mut rotation = TrackRotation { track: Track::Bl1, laps: RaceLaps::Practice, ..Default::default() }.spawn(());
        let mut tick = interval(Duration::from_millis(500));

        loop {
            tokio::select! {
                _result = rotation.poll() => {
                    tracing::info!("Rotation done");
                    break;
                },
                _ = tick.tick() => {
                    tracing::info!("> Lobby interruption");
                }
            }
        }

        tracing::info!("Lobby done");

        Event.call(()).await?;
        Ok(())
    }
}

struct Event;
impl Scene for Event {
    type Output = anyhow::Result<()>;
    type Context = ();

    async fn call(&mut self, _ctx: Self::Context) -> Self::Output {
        for round in 1..=5 {
            tracing::info!("Round {round}/5");
            sleep(Duration::from_secs(1)).await;
        }

        Ok(())
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Setup with a default log level of INFO RUST_LOG is unset
    tracing_subscriber::fmt::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::builder()
                .with_default_directive(tracing_subscriber::filter::LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .init();

    tracing::info!("Hello, world!");

    // temporary bullshit to pretend that we have an insim connection
    let (tx, rx) = sync::watch::channel(10usize);
    let _ = tokio::spawn(async move {
        loop {
            let count = rand::random_range(..47);
            let _ = tx.send(count);
            tracing::info!("[SERVER] Player count changed to {count}");
            sleep(Duration::from_secs(5)).await;
        }
    });

    let ctx = ();

    Supervisor::supervise(ClockworkCarnage, 30, rx).call(ctx).await?;

    Ok(())
}
