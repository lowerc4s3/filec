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

        lock_file(&mut clipboard_file)?;

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
        // TODO: Handle symlinks
        let dest = dest.unwrap_or(Path::new(".")).canonicalize()?;
        let selected = self.get_selected()?;
        let mut failed: Vec<PathBuf> = Vec::with_capacity(selected.len());

        for filename in selected {
            if let Err(e) = move_file(&filename, &dest) {
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

    pub fn get_selected(&self) -> Result<Vec<PathBuf>, FileError> {
        Ok(fs::read_to_string(&self.path)?.lines().map(PathBuf::from).collect())
    }

    pub fn clear(&mut self) -> Result<(), FileError> {
        File::options().write(true).truncate(true).open(&self.path)?;
        Ok(())
    }

    fn add_overwrite<T: AsRef<Path>>(&mut self, files_abs: &[T]) -> Result<(), FileError> {
        let mut clipboard_file =
            BufWriter::new(File::options().write(true).truncate(true).open(&self.path)?);
        for filepath in files_abs {
            writeln!(clipboard_file, "{}", filepath.as_ref().display())?;
        }
        Ok(())
    }
}

fn move_file(filename: &Path, dest: &Path) -> Result<(), MoveFilesError> {
    let new_filename = dest.join(
        filename.file_name().ok_or_else(|| MoveFilesError::Basename(filename.to_path_buf()))?,
    );
    fs::rename(filename, &new_filename).map_err(|e| MoveFilesError::Move {
        from: filename.to_path_buf(),
        to: new_filename,
        source: e,
    })?;
    Ok(())
}

fn lock_file(file: &mut File) -> Result<(), LockError> {
    match file.try_lock_exclusive() {
        Err(e) => Err(LockError{ source: Some(e) }),
        Ok(false) => Err(LockError{ source: None }),
        _ => Ok(()),
    }
}

#[derive(Debug, Error)]
#[error("cannot access clipboard file")]
pub struct FileError(#[from] io::Error);

#[derive(Debug, Error)]
#[error("other process uses clipboard file")]
pub struct LockError {
    source: Option<io::Error>,
}

#[derive(Debug, Error)]
pub enum AddFilesError {
    #[error("cannot open clipboard file")]
    Open(io::Error),

    #[error("cannot process clipboard file")]
    IO(io::Error),

    #[error("cannot add file {} to clipboard file", .filename.display())]
    Add { filename: PathBuf, source: io::Error },

    #[error(transparent)]
    Lock(#[from] LockError),
}

#[derive(Debug, Error)]
pub enum CopyFilesError {}

#[derive(Debug, Error)]
pub enum MoveFilesError {
    #[error("cannot resolve destination dir")]
    GetCWD(#[from] io::Error),

    #[error(transparent)]
    Read(#[from] FileError),

    #[error("cannot get basename of file {}", .0.display())]
    Basename(PathBuf),

    #[error("cannot move {} to {}", .from.display(), .to.display())]
    Move { from: PathBuf, to: PathBuf, source: io::Error },
}
