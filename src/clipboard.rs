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

    pub fn add(&mut self, files: &[impl AsRef<Path>]) -> Result<(), AddError> {
        let mut clip_file = File::options()
            .write(true)
            .read(true)
            .create(true)
            .truncate(false)
            .open(&self.path)
            .map_err(FileError::Access)?;
        utils::lock_file(&mut clip_file)?;

        // Read current contents of clipboard to exclude already added paths
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
            .collect::<Result<HashSet<_>, _>>()?; // Collect into set to remove duplicates

        // Exclude path from clipboard and return error if no files left
        files = &files - &contents;
        if files.is_empty() {
            return Err(AddError::NoNewFiles);
        }

        // File's cursor was moved to the end when we were reading file,
        // no need to modify it
        let mut clip_writer = BufWriter::new(clip_file);
        for path in files {
            writeln!(clip_writer, "{}", path.display()).map_err(FileError::Access)?
        }
        Ok(())
    }

    pub fn copy_to(&mut self, dest: impl AsRef<Path>) -> Result<(), CopyError> {
        todo!();
    }

    pub fn move_to(&mut self, dest: impl AsRef<Path>) -> Result<(), MoveError> {
        // TODO: Handle symlinks
        let dest = dest.as_ref().canonicalize().map_err(|_| DestDirError)?;
        let contents = self.contents()?;

        // Save failed paths so we can leave them in clipboard
        let mut failed: Vec<PathBuf> = Vec::with_capacity(contents.len());

        for filename in contents {
            if let Err(e) = utils::move_file(&filename, &dest) {
                eprintln!("Failed to move {}: {e}", filename.display());
                failed.push(filename);
            }
        }

        if failed.is_empty() {
            // Clear file if operation succeeded...
            self.clear()?;
        } else {
            // ...or leave failed in clipboard
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

    fn add_overwrite(&mut self, files_abs: &[impl AsRef<Path>]) -> Result<(), FileError> {
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

    #[error(transparent)]
    DestPath(#[from] utils::ChangePrefixError),

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

    pub(super) fn move_file(filename: impl AsRef<Path>, dest: impl AsRef<Path>) -> Result<(), MoveError> {
        let new_filename = change_prefix(&filename, &dest)?;
        fs::rename(&filename, &new_filename).map_err(|e| MoveError::Move {
            from: filename.as_ref().to_path_buf(),
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

    fn change_prefix(filename: impl AsRef<Path>, prefix: impl AsRef<Path>) -> Result<PathBuf, ChangePrefixError> {
        let basename = filename
            .as_ref()
            .file_name()
            .ok_or_else(|| ChangePrefixError(filename.as_ref().to_path_buf()))?;
        Ok(prefix.as_ref().join(basename))
    }

    #[derive(Debug, Error)]
    #[error("cannot get {} destination path", .0.display())]
    pub struct ChangePrefixError(PathBuf);

    #[derive(Debug, Error)]
    #[error("other process uses clipboard file")]
    pub struct LockError {
        source: Option<io::Error>,
    }
}
