//! Atomic file writes — write to a temp file, then rename.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use super::error::StoreResult;

/// Atomically write `data` to `path` by first writing to a sibling temp file and then renaming.
///
/// On POSIX systems the rename is atomic. The temp file is cleaned up on failure.
pub fn atomic_write(path: impl AsRef<Path>, data: &[u8]) -> StoreResult<()> {
    let path = path.as_ref();
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent)?;

    let temp_path = temp_sibling(path);
    {
        let mut file = fs::File::create(&temp_path)?;
        file.write_all(data)?;
        file.sync_all().ok();
    }
    fs::rename(&temp_path, path)?;
    Ok(())
}

/// Atomic JSON write with pretty-printing.
pub fn atomic_write_json<T: serde::Serialize>(
    path: impl AsRef<Path>,
    value: &T,
) -> StoreResult<()> {
    let json = serde_json::to_string_pretty(value)
        .map_err(|e| crate::store::error::StoreError::Serialization(e.to_string()))?;
    atomic_write(path, json.as_bytes())
}

fn temp_sibling(path: &Path) -> PathBuf {
    let mut tmp = path.to_path_buf();
    let file_name = tmp
        .file_name()
        .map(|f| f.to_string_lossy().to_string())
        .unwrap_or_else(|| "nexus-cog".into());
    tmp.set_file_name(format!(".{file_name}.tmp"));
    tmp
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn writes_and_reads_back() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("a.json");
        atomic_write(&path, b"{\"x\":1}").unwrap();
        let content = std::fs::read_to_string(&path).unwrap();
        assert_eq!(content, "{\"x\":1}");
    }

    #[test]
    fn no_temp_files_left_behind() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("a.json");
        atomic_write(&path, b"{}").unwrap();
        let entries: Vec<_> = std::fs::read_dir(dir.path()).unwrap().collect();
        let temps: Vec<_> = entries
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_name().to_string_lossy().contains(".tmp"))
            .collect();
        assert!(temps.is_empty());
    }

    #[test]
    fn json_pretty_print_works() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("a.json");
        atomic_write_json(&path, &serde_json::json!({"x": 1, "y": "z"})).unwrap();
        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("\"x\": 1"));
    }
}
