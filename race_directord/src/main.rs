mod cli;
mod config;
mod ecs;
mod web;

pub type Result<T, E = Report> = color_eyre::Result<T, E>;
// A generic error report
// Produced via `Err(some_err).wrap_err("Some context")`
// or `Err(color_eyre::eyre::Report::new(SomeError))`
pub struct Report(color_eyre::Report);

impl std::fmt::Debug for Report {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl<E> From<E> for Report
where
    E: Into<color_eyre::Report>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}

use bevy_ecs::prelude::{Event, EventReader, EventWriter};
use clap::Parser;
use cli::Cli;
use config::{connection::ConnectionConfig, Config, PluginConfig};
use insim::connection::Connection;
use std::{fmt::Debug, io::IsTerminal, path::Path, process::ExitCode, time::Duration};
use tokio::time::{self, sleep};

pub(crate) struct InsimSystem {
    rx: flume::Receiver<insim::connection::Event>,
}

impl InsimSystem {
    pub(crate) fn new(config: &ConnectionConfig) -> Self {
        let (tx, rx) = flume::unbounded();

        let mut conn = config.into_connection();

        tokio::spawn(async move {
            loop {
                match conn.poll().await {
                    Ok(e) => tx.send(e).unwrap(),
                    _ => {
                        tracing::info!("unhandled, sleeping");
                        sleep(Duration::from_secs(1)).await;
                    }
                };
            }
        });

        Self { rx }
    }
}

fn startup() {
    println!("starting up");
}

fn hello_world() {
    println!("hello world!");
}

// This is our event that we will send and receive in systems
#[derive(Event)]
struct MyEvent {
    pub message: String,
    pub random_value: f32,
}

// In every frame we will send an event with a 50/50 chance
fn sending_system(mut event_writer: EventWriter<MyEvent>) {
    let random_value: f32 = rand::random();
    if random_value > 0.5 {
        event_writer.send(MyEvent {
            message: "A random event with value > 0.5".to_string(),
            random_value,
        });
    }
}

// This system listens for events of the type MyEvent
// If an event is received it will be printed to the console
fn receiving_system(mut event_reader: EventReader<MyEvent>) {
    for my_event in event_reader.read() {
        println!(
            "    Received message {:?}, with random value of {}",
            my_event.message, my_event.random_value
        );
    }
}

struct MyEventPlugin;

impl ecs::Plugin for MyEventPlugin {
    fn name(&self) -> &'static str {
        "MyEvent"
    }

    fn register(&self, ecs: &mut ecs::Ecs) {
        ecs.add_event::<MyEvent>();

        ecs.add_system(ecs::PostTick, sending_system);
        ecs.add_system(ecs::Tick, receiving_system);
    }
}

#[tokio::main]
async fn main() -> Result<ExitCode> {
    color_eyre::config::HookBuilder::default()
        .theme(if !std::io::stderr().is_terminal() {
            // Don't attempt color
            color_eyre::config::Theme::new()
        } else {
            color_eyre::config::Theme::dark()
        })
        .install()?;

    let cli = Cli::parse();
    cli.instrumentation.setup()?;

    let config = Config::try_parse(Path::new(&cli.config))?;

    if config.web.enabled {
        web::start(&config.web);
    }

    let mut world = ecs::Ecs::new();
    world.add_system(ecs::Tick, hello_world);
    world.add_system(ecs::Startup, startup);

    println!("{:?}", &config.plugins);

    world.register_plugin(MyEventPlugin);

    world.startup();

    let mut tick = time::interval(Duration::from_millis(500));

    loop {
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                tracing::info!("Requesting shutdown");
                break;
            },

            _ = tick.tick() => {
                world.tick();
            },

        }
    }

    world.shutdown();

    Ok(ExitCode::SUCCESS)
}
