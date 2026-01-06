//! Clockwork carnage. PoC to experiment with the "scene" wrapping idea. go look at
//! twenty_second_league for where this is "going".

use std::{fmt::Debug, marker::PhantomData, time::Duration, usize};

use clap::Parser;
use insim::{core::track::Track, insim::{RaceLaps, TinyType}, WithRequestId};
use kitcar::game::track_rotation::TrackRotation;
use tokio::time::{interval, sleep};

mod cli;

trait HasInsim {
    fn insim(&self) -> insim::builder::SpawnedHandle;
}

trait HasPresence {
    fn presence(&self) -> kitcar::presence::PresenceHandle;
}

trait HasGame {
    fn game(&self) -> kitcar::game::GameHandle;
}

// A stage/layer/scene that orchestrates game flow. long live, delegates to other scene in a
// waterfall manner
trait Scene<C>: Send + Sync + 'static {
    type Output: Debug + Send + Sync + 'static;
    fn poll(&mut self, ctx: C) -> impl Future<Output = Self::Output> + Send;
}

struct Supervisor<L, C>
where
    L: Scene<C> + Clone
{
    inner: L,
    min_players: usize,
    _marker: PhantomData<C>,
}

impl<L: Scene<C> + Clone, C> Supervisor<L, C> {
    fn supervise(inner: L, min_players: usize) -> Self {
        Self {
            inner, min_players, _marker: PhantomData
        }
    }
}

impl<L, C> Scene<C> for Supervisor<L, C>
where 
    L: Scene<C> + Clone,
    C: HasGame + HasPresence + HasInsim + Send + Sync + Clone + 'static
{
    type Output = L::Output;

    async fn poll(&mut self, ctx: C) -> Self::Output {
        loop {
            let min = self.min_players;
            // wait for min_players
            // in the "real world" this would be it's own loop where we also listen for Ncn packets
            // and welcome players
            let presence = ctx.presence();
            let _ = presence.wait_for_player_count(|count| *count > min).await;

            let mut h = tokio::spawn({
                let mut inst = self.inner.clone();
                let ctx = ctx.clone();
                async move { inst.poll(ctx).await }
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
                _ = presence.wait_for_player_count(|val| *val < min) => {
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

impl Scene<Context> for ClockworkCarnage {
    type Output = anyhow::Result<()>;

    async fn poll(&mut self, ctx: Context) -> Self::Output {
        // TODO: handle admin commands

        tracing::info!("Starting...");
        Lobby.poll(ctx).await?;
        Ok(())
    }
}

struct Lobby;
impl Scene<Context> for Lobby {
    type Output = anyhow::Result<()>;

    async fn poll(&mut self, ctx: Context) -> Self::Output {
        tracing::info!("Lobby started");

        let mut rotation = TrackRotation::request(
            ctx.game(), ctx.insim(), Track::Bl1, None, RaceLaps::Practice, None
        );
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

        Event.poll(ctx.clone()).await?;
        Ok(())
    }
}

struct Event;
impl Scene<Context> for Event {
    type Output = anyhow::Result<()>;

    async fn poll(&mut self, _ctx: Context) -> Self::Output {
        for round in 1..=5 {
            tracing::info!("Round {round}/5");
            sleep(Duration::from_secs(1)).await;
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
struct Context {
    pub insim: insim::builder::SpawnedHandle,
    pub presence: kitcar::presence::PresenceHandle,
    pub game: kitcar::game::GameHandle,
}

impl HasPresence for Context {
    fn presence(&self) -> kitcar::presence::PresenceHandle {
        self.presence.clone()
    }
}

impl HasInsim for Context {
    fn insim(&self) -> insim::builder::SpawnedHandle {
        self.insim.clone()
    }
}

impl HasGame for Context {
    fn game(&self) -> kitcar::game::GameHandle {
        self.game.clone()
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

    let args = cli::Args::parse();

    let (insim, _insim_join_handle) = insim::tcp(args.addr.clone())
        .isi_admin_password(args.password.clone())
        .isi_iname("clockwork-carnage".to_owned())
        .isi_prefix('!')
        .spawn(100)
        .await?;

    tracing::info!("Starting clockwork-carnage");

    let ctx = Context {
        insim: insim.clone(),
        presence: kitcar::presence::Presence::spawn(insim.clone(), 32),
        game: kitcar::game::GameInfo::spawn(insim.clone(), 32),
    };

    insim.send(TinyType::Ncn.with_request_id(1)).await?;
    insim.send(TinyType::Npl.with_request_id(2)).await?;
    insim.send(TinyType::Sst.with_request_id(3)).await?;

    Supervisor::supervise(ClockworkCarnage, 30).poll(ctx).await?;

    Ok(())
}
