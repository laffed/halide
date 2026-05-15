use clap::{Parser, Subcommand};

mod commands;
mod config;
mod roll;

#[derive(Parser)]
#[command(name = "halide", about = "Film archive CLI")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Initialize halide config and archive structure
    Init,
    /// Create a new roll archive
    New,
    /// Ingest TIFF scan files into a roll
    Ingest {
        /// Source directory containing TIFF files
        source: Option<String>,
    },
    /// Edit roll or frame metadata
    Note,
    /// Verify archive integrity
    Verify,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Init => commands::init::run(),
        Command::New => commands::new::run(),
        Command::Ingest { source } => commands::ingest::run(source),
        Command::Note => commands::note::run(),
        Command::Verify => commands::verify::run(),
    }
}
