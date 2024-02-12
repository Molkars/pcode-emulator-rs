use std::path::PathBuf;
use clap::{Parser, Subcommand};

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
        #[arg(trailing_var_arg=true, allow_hyphen_values=true)]
        args: Vec<String>
    },
}