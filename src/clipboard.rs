use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::path::{Path, PathBuf};

use thiserror::Error;

#[derive(Debug)]
pub struct Clipboard {
    path: PathBuf,
}

impl Clipboard {
    pub fn new(path: PathBuf) -> Self {
        Clipboard { path }
    }

    // FIXME: Handle duplicate paths
    pub fn add_files<T: AsRef<Path>>(&mut self, files: &[T]) -> Result<(), AddFilesError> {
        let mut clipboard_file = BufWriter::new(
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
            .try_for_each(|path| -> Result<(), AddFilesError> {
                writeln!(clipboard_file, "{}", path?.display())?;
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
    #[error("cannot open clipboard file {}", .filename.display())]
    Open { filename: PathBuf, source: io::Error },
    #[error("cannot add file {} to clipboard file", .filename.display())]
    Add { filename: PathBuf, source: io::Error },
    #[error("cannot write to clipboard file")]
    Write(#[from] io::Error),
}

#[derive(Debug, Error)]
pub enum CopyFilesError {}

#[derive(Debug, Error)]
pub enum MoveFilesError {}
