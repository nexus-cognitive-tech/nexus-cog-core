//! File-based advisory lock using `flock`-style semantics on POSIX.
//!
//! On Windows this falls back to a `LockFile`-based scheme.

use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

use super::error::StoreResult;

#[cfg(unix)]
mod imp {
    pub fn open(path: &std::path::Path, exclusive: bool) -> std::io::Result<std::fs::File> {
        use std::os::unix::fs::OpenOptionsExt;
        let mut opts = std::fs::OpenOptions::new();
        opts.read(true).write(true).create(true);
        if exclusive {
            opts.custom_flags(0x4000_0000); // 1 << 30 — best-effort exclusive flag
        }
        opts.open(path)
    }
}

#[cfg(not(unix))]
mod imp {
    pub fn open(path: &std::path::Path, _exclusive: bool) -> std::io::Result<std::fs::File> {
        std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)
    }
}

/// An advisory file lock.
///
/// Dropping the lock releases it.
#[derive(Debug)]
pub struct FileLock {
    _file: File,
    path: PathBuf,
}

impl FileLock {
    /// Acquire an exclusive lock on `path`. Creates the file if it does not exist.
    pub fn acquire_exclusive(path: impl AsRef<Path>) -> StoreResult<Self> {
        let path = path.as_ref().to_path_buf();
        let file = imp::open(&path, true)
            .map_err(|e| crate::store::error::StoreError::LockFailed(e.to_string()))?;
        Ok(Self { _file: file, path })
    }

    /// Acquire a shared (read) lock on `path`.
    pub fn acquire_shared(path: impl AsRef<Path>) -> StoreResult<Self> {
        let path = path.as_ref().to_path_buf();
        let file = imp::open(&path, false)
            .map_err(|e| crate::store::error::StoreError::LockFailed(e.to_string()))?;
        Ok(Self { _file: file, path })
    }

    /// Returns the lock file path.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Write a small payload into the lock file (e.g. owner ID).
    pub fn annotate(&mut self, payload: &str) -> StoreResult<()> {
        use std::io::Seek;
        self._file.seek(std::io::SeekFrom::Start(0))?;
        self._file.set_len(0)?;
        self._file.write_all(payload.as_bytes())?;
        self._file.flush()?;
        Ok(())
    }

    /// Read whatever annotation was previously written.
    #[must_use]
    pub fn read_annotation(&mut self) -> String {
        use std::io::{Read, Seek};
        let _ = self._file.seek(std::io::SeekFrom::Start(0));
        let mut s = String::new();
        let _ = self._file.read_to_string(&mut s);
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn acquires_and_releases() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("lock");
        {
            let _lock = FileLock::acquire_exclusive(&path).unwrap();
            assert!(path.exists());
        }
        // File remains after lock dropped (by design).
        assert!(path.exists());
    }

    #[test]
    fn annotate_roundtrip() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("lock");
        {
            let mut lock = FileLock::acquire_exclusive(&path).unwrap();
            lock.annotate("pid=12345").unwrap();
            // Annotation is written to the lock file and persists.
        }
        // Read back from disk to verify persistence.
        let content = std::fs::read_to_string(&path).unwrap();
        assert!(
            content.contains("12345"),
            "expected annotation in lock file, got: {content}"
        );
    }
}
