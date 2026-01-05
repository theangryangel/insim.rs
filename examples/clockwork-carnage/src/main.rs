//! Clockwork carnage. PoC to experiment with the "scene" wrapping idea. go look at
//! twenty_second_league for where this is "going".

use std::{fmt::Debug, time::Duration, usize};

use tokio::{sync, time::sleep};

trait Scene: Send + Sync + 'static {
    type Output: Debug + Send + Sync + 'static;
    fn call(&mut self) -> impl Future<Output = Self::Output> + Send;
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

    async fn call(&mut self) -> Self::Output {
        loop {
            let min = self.min_players;
            // wait for min_players
            // in the "real world" this would be it's own loop where we also listen for Ncn packets
            // and welcome players
            let _ = self.rx.wait_for(|val| *val > min).await;

            let mut inst = self.inner.clone();
            let mut h = tokio::spawn(async move { inst.call().await });

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

    async fn call(&mut self) -> Self::Output {
        // TODO: handle admin commands

        tracing::info!("Starting...");
        Lobby.call().await?;
        Ok(())
    }
}

struct Lobby;
impl Scene for Lobby {
    type Output = anyhow::Result<()>;

    async fn call(&mut self) -> Self::Output {
        tracing::info!("Lobby started");
        tokio::time::sleep(Duration::from_secs(2)).await;
        tracing::info!("Lobby done");

        Event.call().await?;
        Ok(())
    }
}

struct Event;
impl Scene for Event {
    type Output = anyhow::Result<()>;

    async fn call(&mut self) -> Self::Output {
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

    Supervisor::supervise(ClockworkCarnage, 30, rx).call().await?;

    Ok(())
}
