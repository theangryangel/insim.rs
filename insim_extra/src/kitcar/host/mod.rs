//! [`Host`]: owns a connection and runs [`Service`]s on it.

mod dispatch;
mod lifecycle;
mod render;
mod run;
#[cfg(test)]
mod tests;

use std::time::{Duration, Instant};

use insim::net::{self, Session, Tokio};
use tokio::{
    sync::{mpsc, oneshot},
    task::JoinHandle,
};

#[cfg(doc)]
use crate::kitcar::{Effects, service::Input};
use crate::{
    kitcar::{
        director::{DirectorCommand, RaceDirector},
        error::HostError,
        service::{AnyMsg, Context, ErasedService, Pools, Service, Typed},
    },
    timers::{self, JobId, Timers},
    ui::Delivery,
    world::World,
};

/// Bound on the outgoing queue and on packets waiting for the host.
const CAPACITY: usize = 256;

/// Buttons are redrawn at most this often, however fast inputs arrive.
const FRAME: Duration = Duration::from_millis(50);

/// A message from work the host started, on its way back.
enum Fired {
    /// From a service's or the director's own timers and tasks.
    Job { owner: u64, job: JobId, msg: AnyMsg },
    /// From a component on one of a service's screens.
    Ui { owner: u64, delivery: Delivery },
}

enum Command {
    Start {
        service: Box<dyn ErasedService>,
        done: oneshot::Sender<()>,
        reply: oneshot::Sender<u64>,
    },
    Stop {
        id: u64,
        reply: oneshot::Sender<()>,
    },
    Direct {
        director: Box<RaceDirector>,
        reply: oneshot::Sender<()>,
    },
    StopDirector {
        reply: oneshot::Sender<()>,
    },
    Shutdown {
        reply: oneshot::Sender<insim::Result<()>>,
    },
}

/// Owns one LFS connection and runs [`Service`]s on it.
///
/// Each packet updates the [`World`] before any service sees it. On start, the
/// host requests the server's state, then delivers
/// [`WorldEvent::Ready`](crate::world::WorldEvent::Ready).
/// Services start and stop without reconnecting, which is how a server manager
/// rotates games:
///
/// ```ignore
/// let host = Host::spawn(session);
/// loop {
///     let service = host.start(next_game(&host).await?).await?;
///     tokio::select! {
///         () = service.finished() => {},
///         () = rotation.changed() => service.stop().await,
///         result = host.closed() => return result,
///     }
/// }
/// ```
///
/// Games get swapped out; licensing or ranking run for good. Every running
/// service sees every input, in order.
#[derive(Debug)]
pub struct Host {
    commands: mpsc::Sender<Command>,
    sender: net::Sender,
    task: Option<JoinHandle<Result<(), HostError>>>,
}

impl Host {
    /// Take over a session, running on the current Tokio runtime.
    pub fn spawn(session: Session<Tokio>) -> Self {
        Self::spawn_with(session, World::new())
    }

    /// Like [`spawn`](Self::spawn), with a configured world such as
    /// [`World::with_rejoin`].
    pub fn spawn_with(session: Session<Tokio>, world: World) -> Self {
        let (sender, receiver, driver) = session.into_split(CAPACITY);
        let driver = tokio::spawn(driver);
        let (commands, command_rx) = mpsc::channel(16);
        let (results_tx, results_rx) = mpsc::unbounded_channel();
        let host = Loop {
            connection: sender.clone(),
            receiver,
            world,
            director: None,
            commands: Vec::new(),
            services: Vec::new(),
            results_tx,
            next_id: 0,
            pools: Pools::new(),
            last_render: timers::now(),
        };
        let task = tokio::spawn(async move {
            let result = host.run(command_rx, results_rx).await;
            match driver.await {
                Ok(Err(error)) => tracing::debug!(%error, "connection driver ended with an error"),
                Ok(Ok(())) => {},
                Err(error) => tracing::error!(%error, "connection driver panicked"),
            }
            result
        });
        Self {
            commands,
            sender,
            task: Some(task),
        }
    }

    /// The connection, for work outside a service.
    pub fn sender(&self) -> &net::Sender {
        &self.sender
    }

    /// Start a service. It gets [`Input::Started`], then every input from then
    /// on.
    pub async fn start<M: Service>(&self, service: M) -> Result<ServiceHandle, HostError> {
        let (reply, id) = oneshot::channel();
        let (done, done_rx) = oneshot::channel();
        self.commands
            .send(Command::Start {
                service: Box::new(Typed::new(service)),
                done,
                reply,
            })
            .await
            .map_err(|_| HostError::Closed)?;
        let id = id.await.map_err(|_| HostError::Closed)?;
        Ok(ServiceHandle {
            id,
            commands: self.commands.clone(),
            done: Some(done_rx),
        })
    }

    /// Hand the races to `director`: lobby setup, restarts and the combo
    /// rotation. It runs before every service, which can read it with
    /// [`Context::director`]. There is only ever one, so this replaces any
    /// director already running (which gets [`Input::Stopping`] first).
    pub async fn start_director(&self, director: RaceDirector) -> Result<(), HostError> {
        let (reply, started) = oneshot::channel();
        self.commands
            .send(Command::Direct {
                director: Box::new(director),
                reply,
            })
            .await
            .map_err(|_| HostError::Closed)?;
        started.await.map_err(|_| HostError::Closed)
    }

    /// Stop the director, if one is running. It gets [`Input::Stopping`], and
    /// from then on services read [`Context::director`] as `None`. The server
    /// is left as it is: whatever track, race or lobby it's on.
    pub async fn stop_director(&self) -> Result<(), HostError> {
        let (reply, stopped) = oneshot::channel();
        self.commands
            .send(Command::StopDirector { reply })
            .await
            .map_err(|_| HostError::Closed)?;
        stopped.await.map_err(|_| HostError::Closed)
    }

    /// Wait until the connection ends, returning its error if it failed.
    /// Services are stopped first. `Err(Closed)` if already awaited.
    pub async fn closed(&mut self) -> Result<(), HostError> {
        match self.task.as_mut() {
            Some(task) => {
                let result = task.await;
                self.task = None;
                result.unwrap_or_else(|error| {
                    tracing::error!(%error, "host panicked");
                    Err(HostError::Closed)
                })
            },
            None => Err(HostError::Closed),
        }
    }

    /// Stop every service (each gets [`Input::Stopping`]), flush their final
    /// packets, send `TINY_CLOSE` and wait for the host to finish.
    pub async fn shutdown(mut self) -> Result<(), HostError> {
        let (reply, result) = oneshot::channel();
        if self
            .commands
            .send(Command::Shutdown { reply })
            .await
            .is_ok()
        {
            let _ = result.await;
        }
        self.closed().await
    }
}

/// A running service. Dropping the handle leaves the service running.
#[derive(Debug)]
pub struct ServiceHandle {
    id: u64,
    commands: mpsc::Sender<Command>,
    done: Option<oneshot::Receiver<()>>,
}

impl ServiceHandle {
    /// Stop the service. It gets [`Input::Stopping`], its final packets are
    /// sent, and its timers and tasks are dropped.
    pub async fn stop(mut self) {
        let (reply, stopped) = oneshot::channel();
        if self
            .commands
            .send(Command::Stop { id: self.id, reply })
            .await
            .is_ok()
        {
            let _ = stopped.await;
        }
        self.finished().await;
    }

    /// Wait until the service stops: by [`Effects::stop`], by being stopped, or
    /// because the connection ended.
    pub async fn finished(&mut self) {
        if let Some(done) = self.done.as_mut() {
            let _ = done.await;
            self.done = None;
        }
    }
}

/// Timers and tasks, kept for anything the host runs. Dropping it aborts
/// them.
struct Slot {
    id: u64,
    timers: Timers<AnyMsg>,
    stopping: bool,
}

impl Slot {
    fn new(id: u64, results: &mpsc::UnboundedSender<Fired>) -> Self {
        let results = results.clone();
        Self {
            id,
            timers: Timers::new(std::sync::Arc::new(move |job, msg| {
                results
                    .send(Fired::Job {
                        owner: id,
                        job,
                        msg,
                    })
                    .is_ok()
            })),
            stopping: false,
        }
    }
}

struct Running {
    slot: Slot,
    service: Box<dyn ErasedService>,
    done: Option<oneshot::Sender<()>>,
}

impl Drop for Running {
    fn drop(&mut self) {
        if let Some(done) = self.done.take() {
            let _ = done.send(());
        }
    }
}

/// The host's one director.
struct Directing {
    slot: Slot,
    director: RaceDirector,
}

/// What services read: the world, and the director's status if there is one.
/// A free function, so callers can still borrow the services mutably.
fn context<'a>(world: &'a World, director: Option<&'a Directing>, now: Instant) -> Context<'a> {
    Context::new(world, now).with_director_opt(director.map(|d| d.director.status()))
}

/// Who an input is for.
#[derive(Debug, Clone, Copy)]
enum Target {
    Director,
    Service(usize),
}

struct Loop {
    connection: net::Sender,
    receiver: net::Receiver,
    world: World,
    director: Option<Directing>,
    /// Commands from services, for the director once their effects are done.
    commands: Vec<DirectorCommand>,
    services: Vec<Running>,
    results_tx: mpsc::UnboundedSender<Fired>,
    next_id: u64,
    pools: Pools,
    last_render: Instant,
}
