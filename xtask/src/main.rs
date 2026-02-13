use clap::{Parser, Subcommand};

mod codepage_lut;

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Generate the insim_core codepage LUT source file
    CodepageLut {
        /// Verify the target file matches generated output without writing
        #[arg(long, default_value_t = false)]
        check: bool,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Command::CodepageLut { check } => codepage_lut::run(check),
    }
}
