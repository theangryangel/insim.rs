use std::{any::Any, time::Duration};

use kitcar::{Context, Engine, Message, Timer, Workshop};

pub struct GameContext {
    pub player_count: u32,
    pub game_state: String,
}

/// A system-specific message to signal shutdown. This isnt actually necessary. But it's a useful
/// demo on how we can use elixir/erlang style mailboxes.
#[derive(Debug)]
struct ShutdownMessage;
impl Message for ShutdownMessage {}

// Simple game server system
#[derive(Debug)]
struct GameServer;

impl Engine<GameContext> for GameServer {
    fn startup(&mut self, _context: &mut Context<GameContext>) {
        println!("'Hello, world!'");
    }
    fn packet(&mut self, _context: &mut Context<GameContext>, packet: &insim::Packet) {
        println!("[Server] 📩 Got message {:?}", packet);
    }
}

#[derive(Debug)]
struct ShutdownTimer;
impl Engine<GameContext> for ShutdownTimer {
    fn handle_message(&mut self, context: &mut Context<GameContext>, message: &dyn Any) {
        if message.is::<ShutdownMessage>() {
            println!("[ShutdownTimer] 🛑 Received shutdown message, triggering shutdown.");
            context.shutdown();
        }
    }
}

// Countdown to shutdown
#[derive(Debug)]
struct CountdownSystem {
    timer: Timer,
    counter: u32,
}

impl CountdownSystem {
    fn new() -> Self {
        Self {
            timer: Timer::repeating(Duration::from_millis(500), Some(5)),
            counter: 0,
        }
    }
}

impl Engine<GameContext> for CountdownSystem {
    fn tick(&mut self, context: &mut Context<GameContext>) {
        if self.timer.tick() {
            self.counter = self.counter.wrapping_add(1);
            println!("[CountdownSystem] Timer tick! Counter: {}", self.counter);
            if self.timer.is_finished() {
                println!(
                    "[CountdownSystem] Timer has finished! Counter: {}",
                    self.counter
                );
                context.send_message(ShutdownMessage);
            }
        }
    }
}

// A new system to demonstrate an infinite timer.
#[derive(Debug)]
struct InfiniteTimerSystem {
    timer: Timer,
    counter: u32,
}

impl InfiniteTimerSystem {
    fn new() -> Self {
        Self {
            timer: Timer::repeating(Duration::from_millis(500), None),
            counter: 0,
        }
    }
}

impl Engine<GameContext> for InfiniteTimerSystem {
    fn tick(&mut self, _context: &mut Context<GameContext>) {
        if self.timer.tick() {
            self.counter = self.counter.wrapping_add(1);
            println!(
                "[InfiniteTimerSystem] Infinite timer tick! Counter: {}",
                self.counter
            );
        }
    }
}

fn main() {
    // Create the context and add it later using the new function
    let game_context = GameContext {
        player_count: 0,
        game_state: "Farts".to_string(),
    };

    Workshop::with_state(game_context)
        // Add systems
        .add_engine(ShutdownTimer)
        .add_engine(GameServer)
        .add_engine(CountdownSystem::new())
        .add_engine(InfiniteTimerSystem::new())
        .ignition(insim::tcp("172.24.64.1:29999").set_non_blocking(true))
        .run(Duration::from_millis(1000 / 60))
}
