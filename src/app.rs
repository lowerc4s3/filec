use std::{
    io::{self, Write},
    path::Path,
};

use anyhow::{Context, Result};

use crate::{Command, clipboard::Clipboard};

pub struct App {
    clipboard: Clipboard,
    cmd: Command,
}

impl App {
    pub fn new(clipboard: Clipboard, cmd: Command) -> Self {
        App { clipboard, cmd }
    }

    pub fn run(&mut self) -> Result<()> {
        match &self.cmd {
            Command::Add(add_args) => {
                self.clipboard.add(&add_args.files).context("failed to add files")
            }
            Command::Copy(copy_args) => {
                let dest = copy_args.dest.as_deref().unwrap_or(Path::new("."));
                self.clipboard.copy_to(dest).context("failed to copy files")
            }
            Command::Move(move_args) => {
                let dest = move_args.dest.as_deref().unwrap_or(Path::new("."));
                self.clipboard.move_to(dest).context("failed to move files")
            }
            Command::List => self.list().context("failed to list contents"),
            Command::Clear => self.clipboard.clear().context("failed to clear clipboard"),
        }
    }

    fn list(&self) -> Result<()> {
        // TODO: Improve formatting
        let mut lock = io::stdout().lock();
        for filename in self.clipboard.contents()? {
            writeln!(lock, "{}", filename.display())?;
        }
        Ok(())
    }
}
