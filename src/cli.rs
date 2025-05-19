use std::path::PathBuf;

use clap::{Parser, Subcommand, Args};

#[derive(Debug, Parser)]
#[command(version, about, propagate_version = true)]
pub struct Cli {
    /// Enable verbose output
    #[arg(short, long)]
    verbose: Option<bool>,
    #[command(subcommand)]
    cmd: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Add files to clipboard
    Add(AddArgs),
    /// Copy files from clipboard to directory
    Copy(CopyArgs),
    /// Copy files from clipboard to directory
    Move(MoveArgs),

    // TODO:
    // List,
    // Drop,
    // List,
    // Help,
    // Exec,
}

#[derive(Debug, Args)]
pub struct AddArgs {
    /// Files to add
    files: Vec<PathBuf>,
}

#[derive(Debug, Args)]
pub struct CopyArgs {
    /// Directory to copy into (cwd by default)
    dest: Option<PathBuf>,
}

#[derive(Debug, Args)]
pub struct MoveArgs {
    /// Directory to move into (cwd by default)
    dest: Option<PathBuf>,
}
