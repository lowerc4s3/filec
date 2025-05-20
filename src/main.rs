use std::{env, fs, path::PathBuf};

use anyhow::{Context, Result};
use buffer::Buffer;
use clap::Parser;
use cli::{Cli, Command};
use directories::ProjectDirs;

mod buffer;
mod cli;

fn main() -> Result<()> {
    let cli = Cli::parse();
    let buffer_path =
        if let Some(user_path) = cli.buffer_path { user_path } else { init_default_buffer_path()? };
    let mut buffer = Buffer::new(buffer_path);
    match &cli.cmd {
        Command::Add(add_args) => buffer.add_files(&add_args.files),
        Command::Copy(copy_args) => buffer.copy_files(copy_args.dest.as_deref()),
        Command::Move(move_args) => buffer.move_files(move_args.dest.as_deref()),
    }
}

fn init_default_buffer_path() -> Result<PathBuf> {
    let buffer_file = get_default_buffer_dir();
    let buffer_dir = buffer_file.parent().expect("default buffer path must have a parent");
    match buffer_dir.try_exists() {
        Err(e) => {
            return Err(e)
                .with_context(|| format!("cannot check existence of {}", buffer_dir.display()));
        }
        Ok(false) => {
            fs::create_dir(buffer_dir)
                .with_context(|| format!("cannot create {}", buffer_dir.display()))?;
        }
        Ok(true) => {}
    }
    Ok(buffer_file)
}

fn get_default_buffer_dir() -> PathBuf {
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
