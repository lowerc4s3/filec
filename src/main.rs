use std::io::Write;
use std::{env, fs, io, path::PathBuf};

use anyhow::{Context, Result};
use clap::Parser;
use cli::{Cli, Command};
use clipboard::Clipboard;
use directories::ProjectDirs;

mod cli;
mod clipboard;

fn main() -> Result<()> {
    let cli = Cli::parse();
    let clipboard_path =
        if let Some(user_path) = cli.clipboard_path { user_path } else { init_clipboard_path()? };
    let mut clipboard = Clipboard::new(clipboard_path);
    match &cli.cmd {
        Command::Add(add_args) => clipboard.add_files(&add_args.files)?,
        Command::Copy(copy_args) => clipboard.copy_files(copy_args.dest.as_deref())?,
        Command::Move(move_args) => clipboard.move_files(move_args.dest.as_deref())?,
        Command::List => {
            // TODO: Improve formatting
            let mut lock = io::stdout().lock();
            for filename in clipboard.get_selected()? {
                writeln!(lock, "{}", filename.display())?;
            }
        }
        Command::Clear => clipboard.clear()?,
    }
    Ok(())
}

fn init_clipboard_path() -> Result<PathBuf> {
    let file = default_clipboard_path();
    let parent = file.parent().expect("default clipboard path must have a parent");
    match parent.try_exists() {
        Err(e) => {
            return Err(e)
                .with_context(|| format!("cannot check existence of {}", parent.display()));
        }
        Ok(false) => {
            fs::create_dir(parent)
                .with_context(|| format!("cannot create {}", parent.display()))?;
        }
        Ok(true) => {}
    }
    Ok(file)
}

fn default_clipboard_path() -> PathBuf {
    // If user defined XDG_DATA_HOME on macOS,
    // use it instead of Application Support
    if cfg!(target_os = "macos") {
        if let Ok(data_dir) = env::var("XDG_DATA_HOME") {
            let mut filec_dir = PathBuf::from(data_dir);
            filec_dir.push("filec");
            filec_dir.push("buf.txt");
            return filec_dir;
        }
    }
    ProjectDirs::from("", "", "filec")
        .expect("cannot get user's home directory")
        .data_dir()
        .with_file_name("buf.txt")
}
