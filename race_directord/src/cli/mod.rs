mod format;
mod instrumentation;

use clap::Parser;

#[derive(Parser)]
pub(crate) enum Command {
    Run,
    GenerateConfig,
}

#[derive(Parser)]
#[command(version)]
#[command(name = "race_directord")]
#[command(about = "A server manager for the Racing Simulator Live for Speed, written in Rust")]
pub(crate) struct Cli {
    /// path to config file
    #[arg(short, long)]
    pub(crate) config: String,

    #[clap(flatten)]
    pub(crate) instrumentation: instrumentation::Instrumentation,
}
