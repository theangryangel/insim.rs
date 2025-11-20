use std::path::PathBuf;

use clap::Parser;

#[derive(Debug, Parser)]
pub struct Args {
    #[arg(short, long, default_value = "config.yaml")]
    pub config_file: PathBuf,
}
