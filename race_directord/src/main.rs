mod cli;
mod config;
mod ecs;
mod web;

mod plugins;

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

use clap::Parser;
use cli::Cli;
use config::Config;
use std::{io::IsTerminal, path::Path, process::ExitCode, time::Duration};
use tokio::time;

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
    world.add_plugin(plugins::insim::Plugin {
        config: config.connection.clone(),
    });

    println!("{:?}", &config.plugins);

    world.add_plugin(plugins::random::Plugin);

    world.add_system(ecs::Tick, plugins::insim::process_insim_mci);

    world.startup();

    let mut tick = time::interval(Duration::from_millis(50));

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
