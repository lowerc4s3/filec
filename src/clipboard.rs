use std::collections::HashSet;
use std::fs::{self, File};
use std::io::{self, BufWriter, Read, Write};
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

    pub fn add<T: AsRef<Path>>(&mut self, files: &[T]) -> Result<(), AddError> {
        let mut clip_file = File::options()
            .write(true)
            .read(true)
            .create(true)
            .truncate(false)
            .open(&self.path)
            .map_err(FileError::Access)?;
        utils::lock_file(&mut clip_file)?;

        let mut buf = String::new();
        clip_file.read_to_string(&mut buf).map_err(FileError::Access)?;
        let contents: HashSet<_> = buf.lines().map(PathBuf::from).collect();

        let mut files = files
            .iter()
            .map(AsRef::as_ref)
            .map(|filename| {
                filename
                    .canonicalize()
                    .map_err(|e| AddError::AbsPath { filename: filename.to_path_buf(), source: e })
            })
            .collect::<Result<HashSet<_>, _>>()?;

        files = &files - &contents;
        if files.is_empty() {
            return Err(AddError::NoNewFiles);
        }

        let mut clip_writer = BufWriter::new(clip_file);
        for path in files {
            writeln!(clip_writer, "{}", path.display()).map_err(FileError::Access)?
        }
        Ok(())
    }

    pub fn copy_to(&mut self, dest: Option<&Path>) -> Result<(), CopyError> {
        todo!();
    }

    pub fn move_to(&mut self, dest: Option<&Path>) -> Result<(), MoveError> {
        // TODO: Handle symlinks
        let dest = dest.unwrap_or(Path::new(".")).canonicalize().map_err(|_| DestDirError)?;
        let selected = self.contents()?;
        let mut failed: Vec<PathBuf> = Vec::with_capacity(selected.len());

        for filename in selected {
            if let Err(e) = utils::move_file(&filename, &dest) {
                eprintln!("Failed to move {}: {e}", filename.display());
                failed.push(filename);
            }
        }

        if failed.is_empty() {
            self.clear()?;
        } else {
            self.add_overwrite(&failed)?;
        }
        Ok(())
    }

    pub fn contents(&self) -> Result<Vec<PathBuf>, FileError> {
        let mut file = File::open(&self.path)?;
        utils::lock_file(&mut file)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        Ok(contents.lines().map(PathBuf::from).collect())
    }

    pub fn clear(&mut self) -> Result<(), FileError> {
        let mut clip_file = File::options().write(true).open(&self.path)?;
        utils::lock_file(&mut clip_file)?;
        clip_file.set_len(0)?;
        Ok(())
    }

    fn add_overwrite<T: AsRef<Path>>(&mut self, files_abs: &[T]) -> Result<(), FileError> {
        let mut clip_file = File::options().write(true).open(&self.path)?;
        utils::lock_file(&mut clip_file)?;
        clip_file.set_len(0)?;
        let mut clip_writer = BufWriter::new(clip_file);
        for filepath in files_abs {
            writeln!(clip_writer, "{}", filepath.as_ref().display())?;
        }
        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum AddError {
    #[error("cannot process clipboard file")]
    File(#[from] FileError),

    #[error("cannot get absolute path of {}", .filename.display())]
    AbsPath { filename: PathBuf, source: io::Error },

    #[error(transparent)]
    Lock(#[from] utils::LockError),

    #[error("no new files to add")]
    NoNewFiles,
}

#[derive(Debug, Error)]
pub enum CopyError {}

#[derive(Debug, Error)]
pub enum MoveError {
    #[error(transparent)]
    DestinationDir(#[from] DestDirError),

    #[error(transparent)]
    Read(#[from] FileError),

    #[error("cannot get basename of file {}", .0.display())]
    Basename(PathBuf),

    #[error("cannot move {} to {}", .from.display(), .to.display())]
    Move { from: PathBuf, to: PathBuf, source: io::Error },
}

#[derive(Debug, Error)]
pub enum FileError {
    #[error("cannot access clipboard file")]
    Access(#[from] io::Error),

    #[error(transparent)]
    Lock(#[from] utils::LockError),
}

#[derive(Debug, Error)]
#[error("cannot resolve destination dir")]
pub struct DestDirError;

mod utils {
    use super::*;
    use fs4::fs_std::FileExt;

    pub(super) fn move_file(filename: &Path, dest: &Path) -> Result<(), MoveError> {
        let new_filename = dest
            .join(filename.file_name().ok_or_else(|| MoveError::Basename(filename.to_path_buf()))?);
        fs::rename(filename, &new_filename).map_err(|e| MoveError::Move {
            from: filename.to_path_buf(),
            to: new_filename,
            source: e,
        })?;
        Ok(())
    }

    pub(super) fn lock_file(file: &mut File) -> Result<(), LockError> {
        match file.try_lock_exclusive() {
            Err(e) => Err(LockError { source: Some(e) }),
            Ok(false) => Err(LockError { source: None }),
            _ => Ok(()),
        }
    }

    #[derive(Debug, Error)]
    #[error("other process uses clipboard file")]
    pub struct LockError {
        source: Option<io::Error>,
    }
}
