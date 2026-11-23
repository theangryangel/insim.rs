//! [`Host`]: owns a connection and runs [`Service`]s on it.

use std::{
    collections::VecDeque,
    time::{Duration, Instant},
};

use insim::{
    Packet,
    net::{self, Session, Tokio},
};
use tokio::{
    sync::{mpsc, oneshot},
    task::{AbortHandle, JoinHandle},
};

use crate::{
    kitcar::{
        Effects,
        director::{DirectorCommand, RaceDirector, RaceDirectorMsg},
        effects::Effect,
        error::HostError,
        service::{AnyMsg, Context, ErasedService, Input, Pools, Service, Typed},
        timers::Timers,
    },
    world::{Applied, SharedWorld, World, WorldEvent, event::WorldReady},
};

/// Bound on the outgoing queue and on packets waiting for the host.
const CAPACITY: usize = 256;

/// Buttons are redrawn at most this often, however fast inputs arrive.
const FRAME: Duration = Duration::from_millis(50);

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
    Shutdown {
        reply: oneshot::Sender<insim::Result<()>>,
    },
}

/// Owns one LFS connection and runs [`Service`]s on it.
///
/// Each packet updates the [`World`] before any service sees it. On start, the
/// host requests the server's state, then delivers [`WorldEvent::Ready`].
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
    world: SharedWorld,
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
        let world = SharedWorld::new(world);
        let (sender, receiver, driver) = session.into_split(CAPACITY);
        let driver = tokio::spawn(driver);
        let (commands, command_rx) = mpsc::channel(16);
        let (results_tx, results_rx) = mpsc::unbounded_channel();
        let startup = {
            let sender = sender.clone();
            tokio::spawn(async move { World::request_startup(&sender).await })
        };
        let host = Loop {
            connection: sender.clone(),
            receiver,
            world: world.clone(),
            director: None,
            commands: Vec::new(),
            services: Vec::new(),
            results_tx,
            ready: Ready::new(startup),
            next_id: 0,
            pools: Pools::new(),
            last_render: Instant::now(),
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
            world,
            task: Some(task),
        }
    }

    /// The connection, for work outside a service, such as attaching a UI
    /// before starting the service that owns it.
    pub fn sender(&self) -> &net::Sender {
        &self.sender
    }

    /// The server mirror.
    pub fn world(&self) -> &SharedWorld {
        &self.world
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

/// Timers and tasks, kept for anything the host runs.
struct Slot {
    id: u64,
    timers: Timers<AnyMsg>,
    tasks: Vec<AbortHandle>,
    stopping: bool,
}

impl Slot {
    fn new(id: u64) -> Self {
        Self {
            id,
            timers: Timers::default(),
            tasks: Vec::new(),
            stopping: false,
        }
    }
}

impl Drop for Slot {
    fn drop(&mut self) {
        for task in &self.tasks {
            task.abort();
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

/// Who an input is for.
#[derive(Debug, Clone, Copy)]
enum Target {
    Director,
    Service(usize),
}

struct Loop {
    connection: net::Sender,
    receiver: net::Receiver,
    world: SharedWorld,
    director: Option<Directing>,
    /// Commands from services, for the director once their effects are done.
    commands: Vec<DirectorCommand>,
    services: Vec<Running>,
    results_tx: mpsc::UnboundedSender<(u64, AnyMsg)>,
    ready: Ready,
    next_id: u64,
    pools: Pools,
    last_render: Instant,
}

impl Loop {
    async fn run(
        mut self,
        mut commands: mpsc::Receiver<Command>,
        mut results: mpsc::UnboundedReceiver<(u64, AnyMsg)>,
    ) -> Result<(), HostError> {
        loop {
            let deadline = self
                .services
                .iter()
                .map(|m| &m.slot)
                .chain(self.director.as_ref().map(|d| &d.slot))
                .filter_map(|slot| slot.timers.deadline())
                .min();
            let waiting = self.ready.task.is_some();
            let wake = match (deadline, self.render_at()) {
                (Some(a), Some(b)) => Some(a.min(b)),
                (a, b) => a.or(b),
            };
            tokio::select! {
                biased;
                command = commands.recv() => match command {
                    Some(Command::Start { service, done, reply }) => {
                        let id = self.next_id;
                        tracing::info!(service = service.name(), "service started");
                        self.next_id += 1;
                        self.services.push(Running {
                            slot: Slot::new(id),
                            service,
                            done: Some(done),
                        });
                        let _ = reply.send(id);
                        self.deliver(Target::Service(self.services.len() - 1), Input::Started).await;
                    },
                    Some(Command::Stop { id, reply }) => {
                        if let Some(index) = self.index_of(id) {
                            self.stop(index).await;
                        }
                        let _ = reply.send(());
                    },
                    Some(Command::Direct { director, reply }) => {
                        tracing::info!("director started");
                        self.stop_director().await;
                        let id = self.next_id;
                        self.next_id += 1;
                        self.director = Some(Directing {
                            slot: Slot::new(id),
                            director: *director,
                        });
                        let _ = reply.send(());
                        self.deliver(Target::Director, Input::Started).await;
                    },
                    Some(Command::Shutdown { reply }) => {
                        self.stop_all().await;
                        let _ = reply.send(self.connection.shutdown().await);
                        return Ok(());
                    },
                    // Every `Host` handle is gone, so nothing can start or stop services.
                    None => {
                        self.stop_all().await;
                        let _ = self.connection.shutdown().await;
                        return Ok(());
                    },
                },
                item = self.receiver.recv() => match item {
                    Some(Ok(packet)) => self.on_packet(packet).await,
                    Some(Err(error @ insim::Error::IncompleteDecode { .. })) => {
                        tracing::warn!(%error, "skipping a partially decoded packet");
                    },
                    Some(Err(error)) => {
                        self.stop_all().await;
                        return Err(error.into());
                    },
                    None => {
                        tracing::info!("connection closed");
                        self.stop_all().await;
                        return Ok(());
                    },
                },
                Some((id, msg)) = results.recv() => {
                    if let Some(target) = self.target_of(id).filter(|&t| self.live(t)) {
                        self.deliver(target, Input::Msg(msg)).await;
                    }
                },
                () = tokio::time::sleep_until(wake.unwrap_or_else(Instant::now).into()), if wake.is_some() => {
                    let now = Instant::now();
                    for target in self.targets() {
                        for msg in self.slot_mut(target).timers.expire(now) {
                            if self.live(target) {
                                self.deliver(target, Input::Msg(msg)).await;
                            }
                        }
                    }
                },
                replies = self.ready.replies(), if waiting => {
                    if let Some(replies) = replies {
                        self.ready.expect(&replies);
                    }
                },
            }
            if self.ready.take() && self.world.write().mark_ready() {
                self.broadcast(|| Input::World(WorldEvent::Ready(WorldReady::default())))
                    .await;
            }
            self.remove_stopped().await;
            if self.render_at().is_some_and(|at| Instant::now() >= at) {
                self.render().await;
            }
        }
    }

    /// When to next redraw: a frame after the last if anything changed, or
    /// when a view asked to be drawn again.
    fn render_at(&self) -> Option<Instant> {
        let frame = self.last_render + FRAME;
        if self.services.iter().any(|s| s.service.needs_render()) {
            return Some(frame);
        }
        self.services
            .iter()
            .filter_map(|s| s.service.redraw_at())
            .min()
            .map(|at| at.max(frame))
    }

    async fn render(&mut self) {
        let now = Instant::now();
        self.last_render = now;
        let mut packets = Vec::new();
        {
            let world = self.world.read();
            let cx = Context::new(&world, now)
                .with_director_opt(self.director.as_ref().map(|d| d.director.status()));
            for running in &mut self.services {
                packets.extend(running.service.render(&cx, &mut self.pools));
            }
        }
        self.send_all(packets).await;
    }

    async fn send_all(&mut self, packets: Vec<Packet>) {
        for packet in packets {
            if let Err(error) = self.connection.send(packet).await {
                tracing::warn!(%error, "failed to send buttons");
            }
        }
    }

    // By index, so no borrow of `services` is held across an await.
    fn index_of(&self, id: u64) -> Option<usize> {
        self.services.iter().position(|m| m.slot.id == id)
    }

    fn target_of(&self, id: u64) -> Option<Target> {
        if self.director.as_ref().is_some_and(|d| d.slot.id == id) {
            return Some(Target::Director);
        }
        self.index_of(id).map(Target::Service)
    }

    /// The director first, then every service in start order.
    fn targets(&self) -> Vec<Target> {
        self.director
            .as_ref()
            .map(|_| Target::Director)
            .into_iter()
            .chain((0..self.services.len()).map(Target::Service))
            .collect()
    }

    fn slot_mut(&mut self, target: Target) -> &mut Slot {
        match target {
            Target::Director => {
                &mut self
                    .director
                    .as_mut()
                    .expect("targets only name a director that is running")
                    .slot
            },
            Target::Service(index) => &mut self.services[index].slot,
        }
    }

    /// Running and not stopping.
    fn live(&self, target: Target) -> bool {
        match target {
            Target::Director => self.director.as_ref().is_some_and(|d| !d.slot.stopping),
            Target::Service(index) => self.services.get(index).is_some_and(|s| !s.slot.stopping),
        }
    }

    async fn on_packet(&mut self, packet: Packet) {
        self.ready.dispatched(&packet);
        let Applied { events, requests } = self.world.write().apply(&packet);
        // Requests the world needs to stay in step.
        for request in requests {
            if let Err(error) = self.connection.send(request).await {
                tracing::warn!(%error, "failed to send a request for the world");
            }
        }
        // The director goes first, so every service reads where it got to.
        for target in self.targets() {
            if !self.live(target) {
                continue;
            }
            self.deliver(target, Input::Packet(packet.clone())).await;
            for event in &events {
                if !self.live(target) {
                    break;
                }
                self.deliver(target, Input::World(event.clone())).await;
            }
        }
        if let Packet::Cnl(cnl) = &packet {
            // Click ids are per connection, so a reused UCID starts fresh.
            let _ = self.pools.remove(&cnl.ucid);
        }
    }

    async fn broadcast(&mut self, input: impl Fn() -> Input<AnyMsg>) {
        for target in self.targets() {
            if self.live(target) {
                self.deliver(target, input()).await;
            }
        }
    }

    async fn deliver(&mut self, target: Target, input: Input<AnyMsg>) {
        let now = Instant::now();
        let effects = {
            let world = self.world.read();
            let cx = Context::new(&world, now);
            match target {
                Target::Director => {
                    let Some(directing) = self.director.as_mut() else {
                        return;
                    };
                    let input = input.map(|msg| {
                        *msg.downcast::<RaceDirectorMsg>()
                            .expect("the host only delivers the director its own messages")
                    });
                    directing
                        .director
                        .update(input, &cx)
                        .map(|msg| Box::new(msg) as AnyMsg)
                },
                Target::Service(index) => {
                    let cx =
                        cx.with_director_opt(self.director.as_ref().map(|d| d.director.status()));
                    self.services[index]
                        .service
                        .update(input, &cx, &mut self.pools)
                },
            }
        };
        self.apply(target, effects, now).await;
        self.direct(now).await;
    }

    /// Hand the director the commands services just gave it.
    async fn direct(&mut self, now: Instant) {
        for command in std::mem::take(&mut self.commands) {
            if !self.live(Target::Director) {
                tracing::warn!(?command, "no director to take this command");
                continue;
            }
            let effects = {
                let world = self.world.read();
                let cx = Context::new(&world, now);
                self.director
                    .as_mut()
                    .expect("checked live")
                    .director
                    .command(command, &cx)
                    .map(|msg| Box::new(msg) as AnyMsg)
            };
            self.apply(Target::Director, effects, now).await;
        }
    }

    async fn apply(&mut self, target: Target, effects: Effects<AnyMsg>, now: Instant) {
        let id = self.slot_mut(target).id;
        let mut queue: VecDeque<_> = effects.items.into();
        while let Some(effect) = queue.pop_front() {
            match effect {
                Effect::Packet(packet) => {
                    if let Err(error) = self.connection.send(packet).await {
                        tracing::warn!(%error, "failed to send a packet from a service");
                    }
                },
                Effect::Timer { key, when, msg } => {
                    self.slot_mut(target)
                        .timers
                        .once(key, when.resolve(now), msg);
                },
                Effect::Every { key, period, make } => {
                    self.slot_mut(target).timers.every(key, period, make, now);
                },
                Effect::Cancel(cancel) => self.slot_mut(target).timers.cancel(&cancel),
                Effect::Task(task) => {
                    let results = self.results_tx.clone();
                    let handle = tokio::spawn(async move {
                        let _ = results.send((id, task.await));
                    });
                    self.track(target, handle.abort_handle());
                },
                Effect::Stream(mut stream) => {
                    let results = self.results_tx.clone();
                    let handle = tokio::spawn(async move {
                        while let Some(msg) = futures::StreamExt::next(&mut stream).await {
                            if results.send((id, msg)).is_err() {
                                break;
                            }
                        }
                    });
                    self.track(target, handle.abort_handle());
                },
                Effect::Send { ucid, key, msg } => {
                    // The director has no view, so nothing to send to.
                    let Target::Service(index) = target else {
                        continue;
                    };
                    let more = {
                        let world = self.world.read();
                        let cx = Context::new(&world, now)
                            .with_director_opt(self.director.as_ref().map(|d| d.director.status()));
                        self.services[index].service.send(ucid, &*key, &*msg, &cx)
                    };
                    // Component replies run before the remaining effects.
                    for effect in more.items.into_iter().rev() {
                        queue.push_front(effect);
                    }
                },
                Effect::Stop => self.slot_mut(target).stopping = true,
                Effect::Director(command) => self.commands.push(command),
            }
        }
    }

    fn track(&mut self, target: Target, task: AbortHandle) {
        let tasks = &mut self.slot_mut(target).tasks;
        tasks.retain(|t| !t.is_finished());
        tasks.push(task);
    }

    async fn stop(&mut self, index: usize) {
        tracing::info!(
            service = self.services[index].service.name(),
            "service stopped"
        );
        self.services[index].slot.stopping = true;
        self.deliver(Target::Service(index), Input::Stopping).await;
        let buttons = self.services[index].service.clear(&mut self.pools);
        self.send_all(buttons).await;
        drop(self.services.remove(index));
    }

    async fn stop_director(&mut self) {
        let Some(directing) = self.director.as_mut() else {
            return;
        };
        tracing::info!("director stopped");
        directing.slot.stopping = true;
        self.deliver(Target::Director, Input::Stopping).await;
        self.director = None;
    }

    /// Services first, then the director they read.
    async fn stop_all(&mut self) {
        while !self.services.is_empty() {
            self.stop(self.services.len() - 1).await;
        }
        self.stop_director().await;
    }

    async fn remove_stopped(&mut self) {
        self.remove_stopped_services().await;
        if self.director.as_ref().is_some_and(|d| d.slot.stopping) {
            self.stop_director().await;
        }
    }

    async fn remove_stopped_services(&mut self) {
        loop {
            let Some(index) = self.services.iter().position(|m| m.slot.stopping) else {
                return;
            };
            self.stop(index).await;
        }
    }
}

/// Holds back [`WorldEvent::Ready`] until every startup reply is in the world.
///
/// A reply future resolves when its packets are *decoded*, which can be before
/// the host has folded them. So the host counts folded packets per ReqI and
/// compares them with the replies once known.
struct Ready {
    task: Option<JoinHandle<insim::Result<Vec<Packet>>>>,
    seen: [usize; 256],
    expected: Option<[usize; 256]>,
}
impl Ready {
    fn new(task: JoinHandle<insim::Result<Vec<Packet>>>) -> Self {
        Self {
            task: Some(task),
            seen: [0; 256],
            expected: None,
        }
    }
    fn dispatched(&mut self, packet: &Packet) {
        if let Some(reqi) = packet.request_id() {
            self.seen[usize::from(reqi.0)] += 1;
        }
    }
    async fn replies(&mut self) -> Option<Vec<Packet>> {
        let task = self.task.as_mut()?;
        let result = task.await;
        self.task = None;
        match result {
            Ok(Ok(replies)) => Some(replies),
            Ok(Err(error)) => {
                tracing::warn!(%error, "startup requests failed; WorldReady will not be delivered");
                None
            },
            Err(error) => {
                tracing::error!(%error, "startup request task panicked");
                None
            },
        }
    }
    fn expect(&mut self, replies: &[Packet]) {
        let mut expected = [0; 256];
        for reqi in replies.iter().filter_map(Packet::request_id) {
            expected[usize::from(reqi.0)] += 1;
        }
        self.expected = Some(expected);
    }
    /// True exactly once, when every startup reply is folded.
    fn take(&mut self) -> bool {
        let done = self
            .expected
            .is_some_and(|expected| expected.iter().zip(&self.seen).all(|(e, s)| s >= e));
        if done {
            self.expected = None;
        }
        done
    }
}

#[cfg(test)]
mod tests {
    use insim::{
        identifiers::ConnectionId,
        insim::{Mtc, Tiny, TinyType},
        test::{Lfs, pair},
    };

    use super::*;

    #[derive(Debug, PartialEq)]
    struct TickKey;

    #[derive(Debug, Clone)]
    enum Msg {
        Tick,
        Loaded(u32),
    }

    /// Announces every step it sees, so the fake LFS can follow along.
    struct Probe(&'static str);

    impl Service for Probe {
        type Msg = Msg;
        fn update(&mut self, input: Input<Msg>, cx: &Context<'_>) -> Effects<Msg> {
            let say = |text: String| Effects::new().message(text, ConnectionId::ALL);
            let name = self.0;
            match input {
                Input::Started => say(format!("{name} started")).schedule_after(
                    TickKey,
                    Duration::from_millis(20),
                    Msg::Tick,
                ),
                Input::Msg(Msg::Tick) => say(format!("{name} tick")).task(async { Msg::Loaded(7) }),
                Input::Msg(Msg::Loaded(n)) => say(format!("{name} loaded {n}")),
                Input::World(WorldEvent::Ready(_)) => {
                    say(format!("{name} ready {}", cx.world().is_ready()))
                },
                Input::Packet(Packet::Tiny(Tiny {
                    subt: TinyType::Ping,
                    ..
                })) => say(format!("{name} pong")).stop(),
                Input::Stopping => say(format!("{name} bye")),
                _ => Effects::none(),
            }
        }
    }

    /// The next chat line from the host, skipping its own startup requests.
    async fn line(lfs: &mut Lfs) -> String {
        loop {
            match lfs.recv().await.expect("host closed the connection") {
                Packet::Mtc(Mtc { text, .. }) => return text,
                Packet::Tiny(Tiny { reqi, .. }) if reqi.0 >= 128 => {},
                other => panic!("unexpected {other:?}"),
            }
        }
    }

    #[tokio::test]
    async fn modes_get_timers_tasks_ready_and_stop_in_order() {
        tokio::time::timeout(Duration::from_secs(10), async {
            let (session, mut lfs) = pair().await;
            let host = Host::spawn(session);
            let service = host.start(Probe("a")).await.unwrap();

            assert_eq!(line(&mut lfs).await, "a started");
            assert_eq!(line(&mut lfs).await, "a tick");
            assert_eq!(line(&mut lfs).await, "a loaded 7");
            // No startup replies: Ready follows once the requests go quiet.
            assert_eq!(line(&mut lfs).await, "a ready true");

            service.stop().await;
            assert_eq!(line(&mut lfs).await, "a bye");

            // A new service on the same connection, without reconnecting.
            let mut service = host.start(Probe("b")).await.unwrap();
            assert_eq!(line(&mut lfs).await, "b started");
            assert_eq!(line(&mut lfs).await, "b tick");
            assert_eq!(line(&mut lfs).await, "b loaded 7");
            // A service can stop itself.
            lfs.send(TinyType::Ping).await;
            assert_eq!(line(&mut lfs).await, "b pong");
            assert_eq!(line(&mut lfs).await, "b bye");
            service.finished().await;

            host.shutdown().await.unwrap();
            let close: Tiny = lfs.expect().await;
            assert_eq!(close.subt, TinyType::Close);
        })
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn a_closed_connection_stops_modes_and_ends_the_host() {
        tokio::time::timeout(Duration::from_secs(10), async {
            let (session, lfs) = pair().await;
            let mut host = Host::spawn(session);
            let mut service = host.start(Probe("a")).await.unwrap();
            drop(lfs);
            service.finished().await;
            // The connection ended rather than being shut down.
            let _ = host.closed().await;
            assert!(matches!(host.closed().await, Err(HostError::Closed)));
            assert!(matches!(
                host.start(Probe("late")).await,
                Err(HostError::Closed)
            ));
        })
        .await
        .unwrap();
    }

    mod director {
        use insim::insim::{Ncn, RaceInProgress, Sta};

        use super::*;
        use crate::kitcar::director::{Combo, Cycle, DirectorCommand};

        /// Reports what it reads of the director.
        struct Watcher;

        impl Service for Watcher {
            type Msg = ();
            fn update(&mut self, input: Input<()>, cx: &Context<'_>) -> Effects<()> {
                let say = |text: String| Effects::new().message(text, ConnectionId::ALL);
                match input {
                    Input::Started => say(format!("watch {}", cx.director().is_some())),
                    Input::Packet(Packet::Tiny(Tiny {
                        subt: TinyType::Ping,
                        ..
                    })) => say(format!(
                        "watch loading {:?}",
                        cx.director().map(|director| director.loading)
                    )),
                    Input::World(WorldEvent::TrackChanged(_)) => say(format!(
                        "watch ready {:?}",
                        cx.director().map(|director| director.ready)
                    )),
                    _ => Effects::none(),
                }
            }
        }

        /// The next chat line from the watcher, skipping the director's own
        /// commands and announcements.
        async fn watched(lfs: &mut Lfs) -> String {
            loop {
                if let Packet::Mtc(Mtc { text, .. }) =
                    lfs.recv().await.expect("host closed the connection")
                    && text.starts_with("watch")
                {
                    return text;
                }
            }
        }

        #[tokio::test]
        async fn services_read_the_director_after_it_has_seen_each_input() {
            tokio::time::timeout(Duration::from_secs(10), async {
                let (session, mut lfs) = pair().await;
                let host = Host::spawn(session);
                let _watcher = host.start(Watcher).await.unwrap();
                assert_eq!(watched(&mut lfs).await, "watch false");

                let fe1x = "FE1X".parse().unwrap();
                host.start_director(RaceDirector::new(Combo::untimed(fe1x)))
                    .await
                    .unwrap();
                let _late = host.start(Watcher).await.unwrap();
                assert_eq!(watched(&mut lfs).await, "watch true");

                // Wait for the first combo to load.
                loop {
                    lfs.send(TinyType::Ping).await;
                    let both = [watched(&mut lfs).await, watched(&mut lfs).await];
                    if both.iter().all(|line| line == "watch loading Some(false)") {
                        break;
                    }
                }

                // A player, then the lobby: the director sets it up, and both
                // watchers see it ready on the same input.
                lfs.send(Ncn {
                    ucid: ConnectionId(1),
                    uname: "driver".into(),
                    ..Default::default()
                })
                .await;
                lfs.send(Sta {
                    track: fe1x,
                    raceinprog: RaceInProgress::No,
                    ..Default::default()
                })
                .await;
                assert_eq!(watched(&mut lfs).await, "watch ready Some(true)");
                assert_eq!(watched(&mut lfs).await, "watch ready Some(true)");

                host.shutdown().await.unwrap();
            })
            .await
            .unwrap();
        }

        /// Asks the director to rotate when pinged.
        struct Rotator;

        impl Service for Rotator {
            type Msg = ();
            fn update(&mut self, input: Input<()>, _: &Context<'_>) -> Effects<()> {
                match input {
                    Input::Packet(Packet::Tiny(Tiny {
                        subt: TinyType::Ping,
                        ..
                    })) => Effects::new().director(DirectorCommand::Rotate),
                    _ => Effects::none(),
                }
            }
        }

        #[tokio::test]
        async fn services_steer_the_director() {
            tokio::time::timeout(Duration::from_secs(10), async {
                let (session, mut lfs) = pair().await;
                let host = Host::spawn(session);
                let combo = |track: &str| Combo::untimed(track.parse().unwrap());
                host.start_director(RaceDirector::new(Cycle::new([combo("FE1"), combo("BL1")])))
                    .await
                    .unwrap();
                let _rotator = host.start(Rotator).await.unwrap();

                lfs.send(TinyType::Ping).await;
                loop {
                    if let Packet::Mtc(Mtc { text, .. }) =
                        lfs.recv().await.expect("host closed the connection")
                        && text.contains("Next track")
                    {
                        assert_eq!(text, "^3Next track: ^7BL1");
                        break;
                    }
                }
                host.shutdown().await.unwrap();
            })
            .await
            .unwrap();
        }
    }

    mod ui {
        use insim::insim::{Bfn, BfnType, Btc, Btn, BtnStyle, Ncn};

        use super::*;
        use crate::ui::{Child, Component, Node, Ui};

        /// A self-contained dialog: opens on request, closes itself when clicked.
        struct Dialog {
            open: bool,
        }
        #[derive(Clone, Debug)]
        enum DialogMsg {
            Show,
            Close,
        }
        impl Component for Dialog {
            type Msg = DialogMsg;
            type Out = ();
            type Props<'a> = ();
            fn update(&mut self, msg: DialogMsg) -> Option<()> {
                self.open = matches!(msg, DialogMsg::Show);
                None
            }
            fn render(&self, (): (), _: &mut Ui<'_, DialogMsg>) -> Node<DialogMsg> {
                if self.open {
                    Node::clickable("help: close", BtnStyle::default(), DialogMsg::Close)
                        .w(30)
                        .h(5)
                } else {
                    Node::empty()
                }
            }
        }

        #[derive(Clone, Debug)]
        enum CounterMsg {
            Bump,
            Help(ConnectionId),
        }

        /// Shared state drawn straight into every player's view.
        struct Counter {
            count: u32,
        }
        impl Service for Counter {
            type Msg = CounterMsg;
            fn update(&mut self, input: Input<CounterMsg>, _: &Context<'_>) -> Effects<CounterMsg> {
                match input {
                    Input::Msg(CounterMsg::Bump) => {
                        self.count += 1;
                        Effects::none()
                    },
                    Input::Msg(CounterMsg::Help(ucid)) => {
                        Effects::new().send(ucid, "help", DialogMsg::Show)
                    },
                    _ => Effects::none(),
                }
            }
            fn view(
                &self,
                ucid: ConnectionId,
                ui: &mut Ui<'_, CounterMsg>,
                _: &Context<'_>,
            ) -> Option<Node<CounterMsg>> {
                Some(
                    Node::container().flex().flex_col().with_children([
                        Node::clickable(
                            format!("count {}", self.count),
                            BtnStyle::default(),
                            CounterMsg::Bump,
                        )
                        .w(30)
                        .h(5),
                        Node::clickable("?", BtnStyle::default(), CounterMsg::Help(ucid))
                            .w(5)
                            .h(5),
                        ui.child(Child::new(|| Dialog { open: false }).key("help")),
                    ]),
                )
            }
        }

        /// The next button packet, skipping the host's startup requests.
        async fn button(lfs: &mut Lfs) -> Packet {
            loop {
                match lfs.recv().await.expect("host closed the connection") {
                    Packet::Tiny(Tiny { reqi, .. }) if reqi.0 >= 128 => {},
                    packet @ (Packet::Btn(_) | Packet::Bfn(_)) => return packet,
                    other => panic!("unexpected {other:?}"),
                }
            }
        }
        async fn btn(lfs: &mut Lfs) -> Btn {
            match button(lfs).await {
                Packet::Btn(btn) => btn,
                other => panic!("expected a button, got {other:?}"),
            }
        }

        #[tokio::test]
        async fn views_draw_state_route_clicks_and_clear_on_stop() {
            tokio::time::timeout(Duration::from_secs(10), async {
                let (session, mut lfs) = pair().await;
                let host = Host::spawn(session);
                let service = host.start(Counter { count: 0 }).await.unwrap();
                let player = ConnectionId(5);
                lfs.send(Ncn {
                    ucid: player,
                    uname: "player".into(),
                    ..Default::default()
                })
                .await;

                let count = btn(&mut lfs).await;
                assert_eq!((count.ucid, count.text.as_str()), (player, "count 0"));
                let help = btn(&mut lfs).await;
                assert_eq!(help.text, "?");

                // The view's own button reaches the service, which redraws.
                lfs.send(Btc {
                    ucid: player,
                    clickid: count.clickid,
                    ..Default::default()
                })
                .await;
                let redrawn = btn(&mut lfs).await;
                assert_eq!(
                    (redrawn.clickid, redrawn.text.as_str()),
                    (count.clickid, "count 1")
                );

                // The service opens the player's dialog by key...
                lfs.send(Btc {
                    ucid: player,
                    clickid: help.clickid,
                    ..Default::default()
                })
                .await;
                let dialog = btn(&mut lfs).await;
                assert_eq!(dialog.text, "help: close");

                // ...and the dialog closes itself.
                lfs.send(Btc {
                    ucid: player,
                    clickid: dialog.clickid,
                    ..Default::default()
                })
                .await;
                assert!(matches!(
                    button(&mut lfs).await,
                    Packet::Bfn(Bfn { subt: BfnType::DelBtn, clickid, .. }) if clickid == dialog.clickid
                ));

                // Stopping removes everything it drew.
                service.stop().await;
                let mut removed = Vec::new();
                for _ in 0..2 {
                    match button(&mut lfs).await {
                        Packet::Bfn(bfn) if bfn.subt == BfnType::DelBtn => {
                            removed.push(bfn.clickid);
                        },
                        other => panic!("expected a removal, got {other:?}"),
                    }
                }
                removed.sort();
                let mut expected = vec![count.clickid, help.clickid];
                expected.sort();
                assert_eq!(removed, expected);
            })
            .await
            .unwrap();
        }
    }
}
