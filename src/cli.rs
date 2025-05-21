use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(version, about, propagate_version = true)]
pub(crate) struct Cli {
    /// Enable verbose output
    #[arg(short, long)]
    pub(crate) verbose: bool,

    /// Provide clipboard file
    #[arg(long, env = "FILEC_CLIPBOARD_PATH")]
    pub(crate) clipboard_path: Option<PathBuf>,

    #[command(subcommand)]
    pub(crate) cmd: Command,
}

#[derive(Debug, Subcommand)]
pub(crate) enum Command {
    /// Add files to clipboard
    Add(AddArgs),

    /// Copy files from clipboard to directory
    #[command(visible_alias("cp"))]
    Copy(CopyArgs),

    /// Copy files from clipboard to directory
    #[command(visible_alias("mv"))]
    Move(MoveArgs),

    /// List selected files
    #[command(visible_alias("ls"))]
    List,

    /// Clear clipboard
    Clear,
    // TODO:
    // Drop,
    // Exec,
}

#[derive(Debug, Args)]
pub(crate) struct AddArgs {
    /// Files to add
    pub(crate) files: Vec<PathBuf>,
}

#[derive(Debug, Args)]
pub(crate) struct CopyArgs {
    /// Directory to copy into (cwd by default)
    pub(crate) dest: Option<PathBuf>,
}

#[derive(Debug, Args)]
pub(crate) struct MoveArgs {
    /// Directory to move into (cwd by default)
    pub(crate) dest: Option<PathBuf>,
}
