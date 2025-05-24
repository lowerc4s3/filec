use fs4::fs_std::FileExt;
use std::collections::HashSet;
use std::fs::{self, File};
use std::io::{self, BufWriter, Read, Seek, SeekFrom, Write};
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

    // TODO: Accept iterator instead of slice
    pub fn add(&mut self, files: &[impl AsRef<Path>]) -> Result<(), ClipboardError> {
        let mut clip_file = ClipboardFile::from_options(
            File::options().write(true).read(true).create(true).truncate(false),
            &self.path,
        )?;
        clip_file.try_lock()?;

        // Collect into set to remove duplicates
        let mut paths: HashSet<_> = files
            .iter()
            .map(AsRef::as_ref)
            .map(|name| name.canonicalize().inspect_err(|e| eprintln!("{}: {e}", name.display())))
            .filter_map(Result::ok)
            .collect();

        if paths.is_empty() {
            return Err(ClipboardError::NoNewFiles);
        }

        // Read current contents of clipboard to exclude already added paths
        let contents: HashSet<_> = clip_file.read_all()?.into_iter().collect();

        // Exclude path from clipboard and return error if no files left
        paths = &paths - &contents;
        if paths.is_empty() {
            return Err(ClipboardError::NoNewFiles);
        }

        // File's cursor was moved to the end when we were reading file,
        // no need to modify it
        clip_file.append(paths.into_iter())?;
        Ok(())
    }

    // TODO: Separate exec logic
    pub fn copy_to(&mut self, dest: &Path) -> Result<(), ClipboardError> {
        let dest = dest.canonicalize().map_err(ClipboardError::DestDir)?;

        let mut clip_file =
            ClipboardFile::from_options(File::options().read(true).write(true), &self.path)?;
        clip_file.try_lock()?;
        let contents = clip_file.read_all()?;

        // Function copy returns errors for nested files
        // so we populate clipboard only with failed files
        let failed: Vec<ExecError> = contents
            .into_iter()
            .filter_map(|name| copy(&name, &dest).err())
            .flatten()
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

    // TODO: Handle symlinks
    pub fn move_to(&mut self, dest: &Path) -> Result<(), ClipboardError> {
        let dest = dest.canonicalize().map_err(ClipboardError::DestDir)?;

        let mut clip_file =
            ClipboardFile::from_options(File::options().read(true).write(true), &self.path)?;
        clip_file.try_lock()?;
        let contents = clip_file.read_all()?;

        // Save failed paths so we can leave them in clipboard
        let errors: Vec<ExecError> = contents
            .into_iter()
            .filter_map(|name| rename(&name, &dest).err())
            .inspect(|e| eprintln!("{e}"))
            .collect();

        clip_file.clear()?;
        if errors.is_empty() {
            Ok(())
        } else {
            // Leave failed files in clipboard
            clip_file.append(errors.into_iter().map(|e| e.file))?;
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
    pub fn open(path: &Path) -> Result<Self, ClipboardFileError> {
        Ok(ClipboardFile(File::open(path)?))
    }

    #[allow(dead_code)]
    pub fn create(path: &Path) -> Result<Self, ClipboardFileError> {
        Ok(ClipboardFile(File::create(path)?))
    }

    pub fn from_options(opts: &fs::OpenOptions, path: &Path) -> Result<Self, ClipboardFileError> {
        Ok(ClipboardFile(opts.open(path)?))
    }

    pub fn clear(&mut self) -> Result<(), ClipboardFileError> {
        self.0.set_len(0)?;
        self.0.seek(SeekFrom::Start(0))?;
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

fn rename(name: &Path, dest: &Path) -> Result<(), ExecError> {
    || -> Result<(), ExecErrorKind> {
        let new_name = dest.join(name.file_name().ok_or(ExecErrorKind::InvalidPath)?);
        fs::rename(name, new_name)?;
        Ok(())
    }()
    .map_err(|e| ExecError { file: name.to_path_buf(), kind: e })
}

// TODO: Mb refactor?
fn copy(name: &Path, dest: &Path) -> Result<(), Vec<ExecError>> {
    let new_name = dest.join(
        name.file_name()
            .ok_or_else(|| vec![ExecError::new(name.into(), ExecErrorKind::InvalidPath)])?,
    );
    let metadata = name.metadata().map_err(|e| vec![ExecError::new(name.into(), e.into())])?;

    if metadata.is_dir() {
        fs::create_dir_all(&new_name).map_err(|e| vec![ExecError::new(dest.into(), e.into())])?;
        let errors: Vec<_> = name
            .read_dir()
            .map_err(|e| vec![ExecError::new(name.into(), e.into())])?
            .filter_map(|res| res.inspect_err(|e| eprintln!("{e}")).ok())
            .filter_map(|entry| copy(&entry.path(), &new_name).err())
            .flatten()
            .collect();
        if !errors.is_empty() { Err(errors) } else { Ok(()) }
    } else {
        fs::copy(name, new_name).map_err(|e| vec![ExecError::new(name.into(), e.into())])?;
        Ok(())
    }
}

#[derive(Debug, Error)]
#[error("{}, {kind}", .file.display())]
pub struct ExecError {
    file: PathBuf,
    kind: ExecErrorKind,
}

impl ExecError {
    pub fn new(file: PathBuf, kind: ExecErrorKind) -> Self {
        ExecError { file, kind }
    }
}

#[derive(Debug, Error)]
pub enum ExecErrorKind {
    #[error("invalid path")]
    InvalidPath,

    #[error(transparent)]
    IO(#[from] io::Error),
}
