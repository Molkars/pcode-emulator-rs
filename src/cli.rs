use std::path::PathBuf;
use clap::{Args, Parser, Subcommand, ValueEnum};

#[derive(Parser, Clone)]
pub struct CLI {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Clone)]
pub enum Command {
    /// Emulate a binary
    Emulate {
        /// the path to the binary
        binary: PathBuf,
    },
}
