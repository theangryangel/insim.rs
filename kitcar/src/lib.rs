//!
use std::{
    io,
    time::{Duration, Instant},
};

use insim::WithRequestId;

/// A trait that defines the behavior and lifecycle of a minigame.
pub trait Engine: Sized {
    /// Error
    type Error: std::fmt::Debug + From<insim::Error>;

    /// Timer Tag
    type TimerTag;

    /// Called once when the game is first initialized.
    /// Use this to set up the initial state of your game.
    fn connected(&mut self, ctx: &mut EngineContext<Self::TimerTag>) -> Result<(), Self::Error>;

    /// Called once when the game is disconnected
    fn disconnected(&mut self, ctx: &mut EngineContext<Self::TimerTag>) -> Result<(), Self::Error>;

    /// Called on every frame or tick. Useful for continuous game logic.
    fn tick(
        &mut self,
        ctx: &mut EngineContext<Self::TimerTag>,
        delta: Duration,
    ) -> Result<(), Self::Error>;

    /// Called on a timer
    fn timer(
        &mut self,
        ctx: &mut EngineContext<Self::TimerTag>,
        tag: Self::TimerTag,
    ) -> Result<(), Self::Error>;

    /// Called whenever a game event occurs.
    /// This is where most of your reactive game logic will live.
    fn packet(
        &mut self,
        ctx: &mut EngineContext<Self::TimerTag>,
        event: &insim::Packet,
    ) -> Result<(), Self::Error>;
}

/// The entry point of the framework.
/// The user calls this from their main function to start the minigame.
pub fn ignite<E: Engine>(con: insim::builder::Builder, mut engine: E) -> Result<(), E::Error> {
    let connection = con.set_non_blocking(true).connect_blocking()?;

    let timers: TimerManager<E::TimerTag> = TimerManager::new();

    let mut ctx = EngineContext { connection, timers };

    engine.connected(&mut ctx)?;

    ctx.connection
        .write(insim::insim::TinyType::Ver.with_request_id(2))?;
    ctx.connection
        .write(insim::insim::TinyType::Ncn.with_request_id(3))?;
    ctx.connection
        .write(insim::insim::TinyType::Npl.with_request_id(4))?;

    let mut last_update = Instant::now();

    loop {
        let completed_timers = ctx.timers.tick();
        for tag in completed_timers {
            engine.timer(&mut ctx, tag)?;
        }

        // consume all pending packets
        // FIXME we probably need a deadline on this
        loop {
            match ctx.connection.read() {
                Ok(packet) => {
                    engine.packet(&mut ctx, &packet)?;
                },
                Err(insim::Error::IO(e)) => {
                    if e.kind() == io::ErrorKind::WouldBlock {
                        // not a real error
                        break;
                    }
                },
                Err(e) => {
                    return Err(e.into());
                },
            }
        }

        // tick
        let now = Instant::now();
        let delta = now.duration_since(last_update);
        last_update = now;
        engine.tick(&mut ctx, delta)?;

        // FIXME - we could have inconsistent tick rates
        std::thread::sleep(Duration::from_millis(16));
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// NewId for Timers
pub struct TimerId(usize);

#[derive(Debug)]
struct Timer<T> {
    id: TimerId,
    target_time: Instant,
    tag: T,
}

#[derive(Debug)]
/// Timer Manager
pub struct TimerManager<T> {
    timers: Vec<Timer<T>>,
    next_id: usize,
}

impl<T> TimerManager<T> {
    /// New
    pub fn new() -> Self {
        Self {
            timers: Vec::new(),
            next_id: 0,
        }
    }

    /// Add a new timer
    pub fn add(&mut self, duration: Duration, tag: T) -> TimerId {
        let id = TimerId(self.next_id);
        self.next_id = self.next_id.wrapping_add(1);
        self.timers.push(Timer {
            id,
            target_time: Instant::now() + duration,
            tag,
        });
        id
    }

    /// Cancel a timer
    pub fn cancel(&mut self, id: TimerId) {
        self.timers.retain(|timer| timer.id != id);
    }

    /// Checks for and returns any completed timer tags.
    pub fn tick(&mut self) -> Vec<T> {
        let now = Instant::now();
        let (completed, active): (Vec<_>, Vec<_>) =
            self.timers.drain(..).partition(|t| now >= t.target_time);

        self.timers = active;
        completed.into_iter().map(|t| t.tag).collect()
    }
}

/// The context object providing access to server functionality
#[derive(Debug)]
pub struct EngineContext<T> {
    /// insim connection
    pub connection: insim::net::blocking_impl::Framed,
    /// timer manager
    pub timers: TimerManager<T>,
    // pub world: WorldManager,
    // pub players: PlayerManager,
    // pub chat: ChatManager,
    // pub ui: UiManager,
    // // ... other managers for timers, etc.
}
