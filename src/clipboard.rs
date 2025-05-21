use std::collections::HashSet;
use std::fs::{self, File};
use std::io::{self, BufWriter, Read, Write};
use std::path::{Path, PathBuf};

use fs4::fs_std::FileExt;
use thiserror::Error;

#[derive(Debug)]
pub struct Clipboard {
    path: PathBuf,
}

impl Clipboard {
    pub fn new(path: PathBuf) -> Self {
        Clipboard { path }
    }

    pub fn add_files<T: AsRef<Path>>(&mut self, files: &[T]) -> Result<(), AddFilesError> {
        let mut clipboard_file = File::options()
            .write(true)
            .read(true)
            .create(true)
            .truncate(false)
            .open(&self.path)
            .map_err(AddFilesError::Open)?;

        match clipboard_file.try_lock_exclusive() {
            Err(e) => return Err(AddFilesError::Lock(Some(e))),
            Ok(false) => return Err(AddFilesError::Lock(None)),
            _ => {}
        }

        let mut buf = String::new();
        clipboard_file.read_to_string(&mut buf).map_err(AddFilesError::IO)?;
        let clipboard_contents: HashSet<_> = buf.lines().map(Path::new).collect();

        let files = files
            .iter()
            .map(AsRef::as_ref)
            .map(|filename| {
                filename
                    .canonicalize()
                    .map_err(|e| AddFilesError::Add { filename: filename.to_path_buf(), source: e })
            })
            .collect::<Result<HashSet<_>, _>>()?;

        // TODO: Return error if there're no new files to add
        let mut clipboard_writer = BufWriter::new(clipboard_file);
        files
            .iter()
            .filter(|filename| !clipboard_contents.contains(filename.as_path()))
            .try_for_each(|path| -> Result<(), _> {
                writeln!(clipboard_writer, "{}", path.display()).map_err(AddFilesError::IO)
            })
    }

    pub fn copy_files(&mut self, dest: Option<&Path>) -> Result<(), AddFilesError> {
        todo!();
    }

    pub fn move_files(&mut self, dest: Option<&Path>) -> Result<(), MoveFilesError> {
        todo!();
    }

    pub fn get_selected(&self) -> Result<Vec<PathBuf>, GetSelectedError> {
        Ok(fs::read_to_string(&self.path)?.lines().map(PathBuf::from).collect())
    }
}

#[derive(Debug, Error)]
pub enum AddFilesError {
    #[error("cannot open clipboard file")]
    Open(io::Error),

    #[error("cannot process clipboard file")]
    IO(io::Error),

    #[error("cannot add file {} to clipboard file", .filename.display())]
    Add { filename: PathBuf, source: io::Error },

    #[error("other process uses clipboard")]
    Lock(Option<io::Error>),
}

#[derive(Debug, Error)]
pub enum CopyFilesError {}

#[derive(Debug, Error)]
pub enum MoveFilesError {}

#[derive(Debug, Error)]
#[error("cannot read clipboard contents")]
pub struct GetSelectedError(#[from] io::Error);
