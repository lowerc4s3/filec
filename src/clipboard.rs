use fs4::fs_std::FileExt;
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

    pub fn add(&mut self, files: &[impl AsRef<Path>]) -> Result<(), ClipboardError> {
        let mut clip_file = ClipboardFile::from_options(
            File::options().write(true).read(true).create(true).truncate(false),
            &self.path,
        )?;
        clip_file.try_lock()?;

        let mut failed = false;

        // Collect into set to remove duplicates
        let mut paths: HashSet<_> = files
            .iter()
            .map(AsRef::as_ref)
            .map(|name| name.canonicalize().inspect_err(|e| eprintln!("{}: {e}", name.display())))
            .inspect(|res| failed |= res.is_err())
            .filter_map(Result::ok)
            .collect();
        paths.is_empty().then_some(()).ok_or(ClipboardError::NoNewFiles)?;

        // Read current contents of clipboard to exclude already added paths
        let contents: HashSet<_> = clip_file.read_all()?.into_iter().collect();

        // Exclude path from clipboard and return error if no files left
        paths = &paths - &contents;
        paths.is_empty().then_some(()).ok_or(ClipboardError::NoNewFiles)?;

        // File's cursor was moved to the end when we were reading file,
        // no need to modify it
        clip_file.append(paths.into_iter())?;

        // Return error if some files were processed with errors
        failed.then_some(()).ok_or(ClipboardError::PartitialFail)
    }

    pub fn copy_to(&mut self, dest: impl AsRef<Path>) -> Result<(), ClipboardError> {
        todo!();
    }

    pub fn move_to(&mut self, dest: impl AsRef<Path>) -> Result<(), ClipboardError> {
        // TODO: Handle symlinks
        let dest = dest.as_ref().canonicalize().map_err(ClipboardError::DestDir)?;

        let mut clip_file = ClipboardFile::open(&self.path)?;
        clip_file.try_lock()?;
        let contents = clip_file.read_all()?;

        // Save failed paths so we can leave them in clipboard
        let failed: Vec<MoveError> = contents
            .into_iter()
            .filter_map(|name| move_file(&name, &dest).err())
            .inspect(|e| eprintln!("{e}"))
            .collect();

        clip_file.clear()?;
        if failed.is_empty() {
            Ok(())
        } else {
            // Leave failed files in clipboard
            clip_file.append(failed.into_iter().map(|e| e.file))?;
            Err(ClipboardError::PartitialFail)
        }
    }

    pub fn contents(&self) -> Result<Vec<PathBuf>, ClipboardError> {
        let mut clip_file = ClipboardFile::open(&self.path)?;
        clip_file.try_lock()?;
        Ok(clip_file.read_all()?)
    }

    pub fn clear(&mut self) -> Result<(), ClipboardError> {
        let mut clip_file = ClipboardFile::from_options(File::options().write(true), &self.path)?;
        clip_file.try_lock()?;
        clip_file.clear()?;
        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum ClipboardError {
    #[error("cannot access clipboard file")]
    ClipboardFile(#[from] ClipboardFileError),

    #[error("there's no new files to add")]
    NoNewFiles,

    #[error("cannot access destination dir")]
    DestDir(#[source] io::Error),

    #[error("one or more files were processed with errors")]
    PartitialFail,
}

#[derive(Debug)]
struct ClipboardFile(File);

impl ClipboardFile {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, ClipboardFileError> {
        Ok(ClipboardFile(File::open(path)?))
    }

    #[allow(dead_code)]
    pub fn create(path: impl AsRef<Path>) -> Result<Self, ClipboardFileError> {
        Ok(ClipboardFile(File::create(path)?))
    }

    pub fn from_options(
        opts: &fs::OpenOptions,
        path: impl AsRef<Path>,
    ) -> Result<Self, ClipboardFileError> {
        Ok(ClipboardFile(opts.open(path)?))
    }

    pub fn clear(&mut self) -> Result<(), ClipboardFileError> {
        self.0.set_len(0)?;
        Ok(())
    }

    pub fn try_lock(&self) -> Result<(), ClipboardFileError> {
        match self.0.try_lock_exclusive() {
            Err(e) => Err(ClipboardFileError::Lock(Some(e))),
            Ok(false) => Err(ClipboardFileError::Lock(None)),
            _ => Ok(()),
        }
    }

    pub fn read_all(&mut self) -> Result<Vec<PathBuf>, ClipboardFileError> {
        let mut contents = String::new();
        self.0.read_to_string(&mut contents)?;
        Ok(contents.lines().map(PathBuf::from).collect())
    }

    pub fn append(
        &mut self,
        filenames: impl Iterator<Item = impl AsRef<Path>>,
    ) -> Result<(), ClipboardFileError> {
        let mut writer = BufWriter::new(&self.0);
        for name in filenames {
            writeln!(writer, "{}", name.as_ref().display()).map_err(ClipboardFileError::Access)?;
        }
        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum ClipboardFileError {
    #[error("other process acquired lock on clipboard file")]
    Lock(#[source] Option<io::Error>),

    #[error("cannot access clipboard file")]
    Access(#[from] io::Error),
}

fn move_file(name: impl AsRef<Path>, dest: impl AsRef<Path>) -> Result<(), MoveError> {
    let new_name = dest.as_ref().join(
        name.as_ref()
            .file_name()
            .ok_or_else(|| MoveError { file: name.as_ref().to_path_buf(), source: None })?,
    );
    fs::rename(&name, &new_name)
        .map_err(|e| MoveError { file: name.as_ref().to_path_buf(), source: Some(e) })?;
    Ok(())
}

#[derive(Debug, Error)]
#[error("{}: {}", .file.display(), .source.as_ref().unwrap_or(&io::Error::other("Invalid path")))] // NOTE: ???
pub struct MoveError {
    file: PathBuf,
    source: Option<io::Error>,
}
