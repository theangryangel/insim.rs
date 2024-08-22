use std::path;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use insim_pth::Pth;

mod complex;
mod simple;

const SCALE: f32 = 65536.0;

#[derive(Debug, Subcommand)]
enum Mode {
    Simple(simple::SimpleArgs),
    Complex(complex::ComplexArgs),
}

/// pth2svg converts one or more PTH files to a simplified SVG image
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    mode: Mode,

    #[clap(long, default_value_t = 20.0)]
    viewbox_padding: f32,

    #[clap(short, long)]
    output: path::PathBuf,

    #[clap(short, long, default_value_t = false)]
    force: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    if args.output.exists() {
        if !args.force {
            let err = Err(std::io::Error::from(std::io::ErrorKind::AlreadyExists));
            return err.context(format!("Output path already exists {:?}", &args.output));
        }

        std::fs::remove_file(&args.output)?;
    }

    let document = match args.mode {
        Mode::Simple(c) => c.run(args.viewbox_padding)?,
        Mode::Complex(c) => c.run(args.viewbox_padding)?,
    };

    svg::save(&args.output, &document)
        .context(format!("Could not save output SVG to '{:?}'", &args.output))?;

    Ok(())
}
