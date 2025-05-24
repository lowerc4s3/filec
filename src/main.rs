use std::{env, fs, path::PathBuf};

use anyhow::{Context, Result};
use app::App;
use clap::Parser;
use cli::{Cli, Command};
use clipboard::Clipboard;
use directories::ProjectDirs;

mod app;
mod cli;
mod clipboard;

fn main() -> Result<()> {
    let cli = Cli::parse();
    let clip_path = cli.clipboard_path.ok_or(()).or_else(|()| init_default())?;
    let mut app = App::new(Clipboard::new(clip_path), cli.cmd);
    app.run()
}

fn init_default() -> Result<PathBuf> {
    let file = default_path()?;
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

fn default_path() -> Result<PathBuf> {
    // If user defined XDG_DATA_HOME on macOS,
    // use it instead of Application Support
    if cfg!(target_os = "macos") {
        if let Ok(data_dir) = env::var("XDG_DATA_HOME") {
            let mut default_dir = PathBuf::from(data_dir);
            default_dir.push("filec");
            default_dir.push("buf.txt");
            return Ok(default_dir);
        }
    }
    Ok(ProjectDirs::from("", "", "filec")
        .context("cannot get user's home directory")?
        .data_dir()
        .with_file_name("buf.txt"))
}
