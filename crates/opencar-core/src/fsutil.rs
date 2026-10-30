// SPDX-License-Identifier: GPL-3.0-or-later
//! File helpers shared by every module that saves something.

use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

/// Writes a file so that a power cut leaves either the old or the new content, never half of it.
///
/// The bytes go to a temporary file next to the target, which is flushed to disk and renamed over
/// the target. When `backup` is true and the target exists, its previous content is first copied to
/// `<name>.bak`. Missing parent directories are created.
pub fn write_atomic(path: &Path, bytes: &[u8], backup: bool) -> io::Result<()> {
    let dir = path.parent().filter(|p| !p.as_os_str().is_empty()).unwrap_or(Path::new("."));
    fs::create_dir_all(dir)?;
    if backup && path.exists() {
        fs::copy(path, backup_path(path))?;
    }
    let name = path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
    let tmp = dir.join(format!(".{name}.tmp-{}", std::process::id()));
    {
        let mut file = fs::File::create(&tmp)?;
        file.write_all(bytes)?;
        file.sync_all()?;
    }
    if let Err(e) = fs::rename(&tmp, path) {
        let _ = fs::remove_file(&tmp);
        return Err(e);
    }
    if let Ok(d) = fs::File::open(dir) {
        let _ = d.sync_all();
    }
    Ok(())
}

/// The backup path `write_atomic` uses for a file: the same name with `.bak` appended.
pub fn backup_path(path: &Path) -> PathBuf {
    let mut name = path.file_name().unwrap_or_default().to_os_string();
    name.push(".bak");
    path.with_file_name(name)
}

/// Reads a small text file, trimmed; `None` when it is missing or unreadable.
pub fn read_trimmed(path: &Path) -> Option<String> {
    fs::read_to_string(path).ok().map(|s| s.trim_matches(|c: char| c.is_whitespace() || c == '\0').to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn writes_and_keeps_a_backup() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("sub/config.json");
        write_atomic(&path, b"one", true).unwrap();
        assert_eq!(fs::read_to_string(&path).unwrap(), "one");
        assert!(!backup_path(&path).exists());
        write_atomic(&path, b"two", true).unwrap();
        assert_eq!(fs::read_to_string(&path).unwrap(), "two");
        assert_eq!(fs::read_to_string(backup_path(&path)).unwrap(), "one");
    }

    #[test]
    fn leaves_no_temporary_files() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("a.toml");
        write_atomic(&path, b"x", false).unwrap();
        let names: Vec<_> = fs::read_dir(dir.path()).unwrap().map(|e| e.unwrap().file_name()).collect();
        assert_eq!(names, vec![std::ffi::OsString::from("a.toml")]);
    }

    #[test]
    fn read_trimmed_strips_nul_and_newlines() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("model");
        fs::write(&path, b"Raspberry Pi 5 Model B Rev 1.0\0\n").unwrap();
        assert_eq!(read_trimmed(&path).unwrap(), "Raspberry Pi 5 Model B Rev 1.0");
        assert_eq!(read_trimmed(&dir.path().join("none")), None);
    }
}
