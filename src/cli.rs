use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(
    name = "memos-rs",
    version,
    about = "Rust backend foundation for a self-hostable Memos replacement"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    Serve(ServeArgs),
}

#[derive(Debug, Args)]
pub struct ServeArgs {
    #[arg(long, default_value = "config.toml")]
    pub config: PathBuf,
}
