use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::path::{Path, PathBuf};

use thiserror::Error;

#[derive(Debug)]
pub struct Buffer {
    path: PathBuf,
}

impl Buffer {
    pub fn new(path: PathBuf) -> Self {
        Buffer { path }
    }

    // FIXME: Handle duplicate paths
    pub fn add_files<T: AsRef<Path>>(&mut self, files: &[T]) -> Result<(), AddFilesError> {
        let mut buffer_file = BufWriter::new(
            File::options()
                .create(true)
                .append(true)
                .open(&self.path)
                .map_err(|e| AddFilesError::Open { filename: self.path.clone(), source: e })?,
        );
        files
            .iter()
            .map(|filename| {
                filename.as_ref().canonicalize().map_err(|e| AddFilesError::Add {
                    filename: filename.as_ref().to_path_buf(),
                    source: e,
                })
            })
            .try_for_each(|filename| -> Result<(), AddFilesError> {
                writeln!(buffer_file, "{}", filename?.display())?;
                Ok(())
            })?;
        Ok(())
    }

    pub fn copy_files(&mut self, dest: Option<&Path>) -> Result<(), AddFilesError> {
        todo!();
    }

    pub fn move_files(&mut self, dest: Option<&Path>) -> Result<(), MoveFilesError> {
        todo!();
    }
}

#[derive(Debug, Error)]
pub enum AddFilesError {
    #[error("cannot open buffer {}", .filename.display())]
    Open { filename: PathBuf, source: io::Error },
    #[error("cannot add file {} to buffer", .filename.display())]
    Add { filename: PathBuf, source: io::Error },
    #[error("cannot write to buffer")]
    Write(#[from] io::Error),
}

#[derive(Debug, Error)]
pub enum CopyFilesError {}

#[derive(Debug, Error)]
pub enum MoveFilesError {}
