use std::{env, fs, path::PathBuf};

use anyhow::{Context, Result};
use clap::Parser;
use cli::Cli;
use directories::ProjectDirs;

mod cli;

fn main() -> Result<()> {
    let args = Cli::parse();
    let buf_file = if let Some(user_path) = args.buffer_path {
        user_path
    } else {
        init_default_buffer_path()?
    };
    Ok(())
}

fn init_default_buffer_path() -> Result<PathBuf> {
    let buf_file = get_default_buffer_dir();
    let buf_dir = buf_file.parent().expect("default buffer path must have a parent");
    match buf_dir.try_exists() {
        Err(e) => {
            return Err(e)
                .with_context(|| format!("cannot check existence of {}", buf_dir.display()));
        }
        Ok(false) => {
            fs::create_dir(buf_dir).with_context(|| format!("cannot create {}", buf_dir.display()))?;
        }
        Ok(true) => {}
    }
    Ok(buf_file)
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
