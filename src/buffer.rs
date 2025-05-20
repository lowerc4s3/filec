use std::path::{Path, PathBuf};

use anyhow::Result;

pub(crate) struct Buffer {
    path: PathBuf,
}

impl Buffer {
    pub fn new(path: PathBuf) -> Self {
        Buffer { path }
    }
    pub fn add_files(&mut self, files: &[PathBuf]) -> Result<()> {
        todo!();
    }
    pub fn copy_files(&mut self, dest: Option<&Path>) -> Result<()> {
        todo!();
    }
    pub fn move_files(&mut self, dest: Option<&Path>) -> Result<()> {
        todo!();
    }
}
