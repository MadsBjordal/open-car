// SPDX-License-Identifier: GPL-3.0-or-later
//! The compatibility database: every `compat/profiles/*.toml`, loaded, checked and searchable.

use crate::profile::{DeviceProfile, Problem};
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::Path;

/// All device profiles, each with the name of the file it came from.
#[derive(Debug, Default, Clone)]
pub struct CompatDb {
    /// `(file name, profile)` pairs, sorted by file name.
    pub entries: Vec<(String, DeviceProfile)>,
    /// Files that could not be read as a profile: `(file name, error)`.
    pub unreadable: Vec<(String, String)>,
}

impl CompatDb {
    /// Builds the database from `(file name, TOML text)` pairs, e.g. the profiles built into the binary.
    pub fn from_sources<'a>(sources: impl IntoIterator<Item = (&'a str, &'a str)>) -> Self {
        let mut db = CompatDb::default();
        for (name, text) in sources {
            match DeviceProfile::from_toml(text) {
                Ok(p) => db.entries.push((name.to_string(), p)),
                Err(e) => db.unreadable.push((name.to_string(), e.to_string())),
            }
        }
        db.entries.sort_by(|a, b| a.0.cmp(&b.0));
        db
    }

    /// Loads every `*.toml` file in a folder.
    pub fn load_dir(dir: &Path) -> io::Result<Self> {
        let mut files = Vec::new();
        for entry in fs::read_dir(dir)? {
            let path = entry?.path();
            if path.extension().is_some_and(|e| e == "toml") {
                let name = path.file_name().unwrap_or_default().to_string_lossy().into_owned();
                files.push((name, fs::read_to_string(&path)?));
            }
        }
        Ok(Self::from_sources(files.iter().map(|(n, t)| (n.as_str(), t.as_str()))))
    }

    /// Every problem in the database: unreadable files, broken rules, ids not matching the
    /// file name, duplicate ids. Empty = the database is valid.
    pub fn validate(&self) -> Vec<(String, Problem)> {
        let mut out: Vec<(String, Problem)> =
            self.unreadable.iter().map(|(f, e)| (f.clone(), Problem(format!("cannot be read: {e}")))).collect();
        let mut seen: HashMap<&str, &str> = HashMap::new();
        for (file, p) in &self.entries {
            for problem in p.validate() {
                out.push((file.clone(), problem));
            }
            if file.strip_suffix(".toml") != Some(p.id.as_str()) {
                out.push((file.clone(), Problem(format!("id '{}' must match the file name", p.id))));
            }
            if let Some(other) = seen.insert(p.id.as_str(), file.as_str()) {
                out.push((file.clone(), Problem(format!("id '{}' is also used by {other}", p.id))));
            }
        }
        out
    }

    /// Profiles whose id, maker, model or vehicles contain every word of `query` (any case).
    /// An empty query gives every profile.
    pub fn search(&self, query: &str) -> Vec<&DeviceProfile> {
        let words: Vec<String> = query.split_whitespace().map(str::to_lowercase).collect();
        self.entries
            .iter()
            .map(|(_, p)| p)
            .filter(|p| {
                let text = p.search_text();
                words.iter().all(|w| text.contains(w.as_str()))
            })
            .collect()
    }

    /// The profile with this id.
    pub fn get(&self, id: &str) -> Option<&DeviceProfile> {
        self.entries.iter().map(|(_, p)| p).find(|p| p.id == id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const A: &str = r#"
id = "scion-frs-pioneer"
manufacturer = "Pioneer"
model = "Scion head unit"
vehicles = ["Scion FR-S 2013-2016", "Toyota 86"]
installation = "unknown"
android_auto = "unknown"
carplay = "unknown"
status = "untested"
"#;
    const B: &str = r#"
id = "raspberry-pi-5"
manufacturer = "Raspberry Pi"
model = "Pi 5"
installation = "external-adapter"
android_auto = "unknown"
carplay = "unknown"
status = "untested"
"#;

    #[test]
    fn search_matches_all_words_in_any_case() {
        let db = CompatDb::from_sources([("scion-frs-pioneer.toml", A), ("raspberry-pi-5.toml", B)]);
        assert_eq!(db.search("scion fr-s").len(), 1);
        assert_eq!(db.search("TOYOTA 86")[0].id, "scion-frs-pioneer");
        assert_eq!(db.search("pi").len(), 2);
        assert_eq!(db.search("").len(), 2);
        assert!(db.search("brz").is_empty());
        assert!(db.get("raspberry-pi-5").is_some());
    }

    #[test]
    fn validate_finds_name_mismatch_duplicates_and_unreadable_files() {
        let db = CompatDb::from_sources([("scion-frs-pioneer.toml", A), ("copy.toml", A), ("broken.toml", "id = ")]);
        let problems: Vec<String> = db.validate().iter().map(|(f, p)| format!("{f}: {p}")).collect();
        assert!(problems.iter().any(|p| p.starts_with("broken.toml: cannot be read")));
        assert!(problems.iter().any(|p| p.contains("copy.toml: id 'scion-frs-pioneer' must match")));
        assert!(problems.iter().any(|p| p.contains("also used by")));
        assert_eq!(problems.len(), 3);
    }

    #[test]
    fn load_dir_reads_only_toml_files() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("raspberry-pi-5.toml"), B).unwrap();
        fs::write(dir.path().join("README.md"), "not a profile").unwrap();
        let db = CompatDb::load_dir(dir.path()).unwrap();
        assert_eq!(db.entries.len(), 1);
        assert!(db.validate().is_empty());
    }
}
