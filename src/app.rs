use std::io::{self, Write};

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
                self.clipboard.add(&add_args.files).context("Failed to add files")
            }
            Command::Copy(copy_args) => {
                self.clipboard.copy_to(copy_args.dest.as_deref()).context("Failed to copy files")
            }
            Command::Move(move_args) => {
                self.clipboard.move_to(move_args.dest.as_deref()).context("Failed to move files")
            }
            Command::List => self.list().context("Failed to list contents"),
            Command::Clear => self.clipboard.clear().context("Failed to clear clipboard"),
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
