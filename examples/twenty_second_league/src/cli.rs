use std::path::PathBuf;

use clap::{Parser};

#[derive(Debug, Parser)]
pub struct Args {
    #[arg(short, long, default_value="20s.db")]
    pub database: PathBuf,

    #[arg(long)]
    pub addr: String,

    #[arg(long)]
    pub admin: Option<String>,
}
