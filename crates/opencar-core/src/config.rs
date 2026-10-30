// SPDX-License-Identifier: GPL-3.0-or-later
//! open-car's own settings (`~/.config/open-car/config.toml`). LIVI keeps its own settings;
//! these only say which hardware profile this box is and how CarPlay authenticates here.

use crate::fsutil::write_atomic;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// open-car's settings on this computer.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// Id of the hardware profile in the compatibility database that describes this computer.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub profile: Option<String>,
    /// A LIVI Link box is connected (brings its own radios and CarPlay authentication).
    #[serde(default)]
    pub livi_link: bool,
    /// I²C bus an MFi coprocessor is wired to, if one is fitted.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mfi_i2c_bus: Option<u32>,
}

/// Errors reading or writing the settings.
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    /// The file could not be read or written.
    #[error("{0}: {1}")]
    Io(PathBuf, std::io::Error),
    /// The file is not valid settings.
    #[error("{0} is not valid: {1}")]
    Parse(PathBuf, toml::de::Error),
}

/// Where the settings live: `$XDG_CONFIG_HOME/open-car/config.toml`, else `$HOME/.config/open-car/config.toml`.
pub fn config_path(xdg_config_home: Option<&str>, home: Option<&str>) -> Option<PathBuf> {
    let base = match (xdg_config_home.filter(|s| !s.is_empty()), home.filter(|s| !s.is_empty())) {
        (Some(x), _) => PathBuf::from(x),
        (None, Some(h)) => Path::new(h).join(".config"),
        (None, None) => return None,
    };
    Some(base.join("open-car/config.toml"))
}

impl Config {
    /// Reads the settings; a missing file gives the defaults.
    pub fn load(path: &Path) -> Result<Self, ConfigError> {
        match std::fs::read_to_string(path) {
            Ok(text) => toml::from_str(&text).map_err(|e| ConfigError::Parse(path.into(), e)),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Self::default()),
            Err(e) => Err(ConfigError::Io(path.into(), e)),
        }
    }

    /// Saves the settings safely against power loss, keeping the previous file as `.bak`.
    pub fn save(&self, path: &Path) -> Result<(), ConfigError> {
        let text = toml::to_string_pretty(self).expect("settings always serialise");
        write_atomic(path, text.as_bytes(), true).map_err(|e| ConfigError::Io(path.into(), e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn path_follows_xdg_then_home() {
        assert_eq!(config_path(Some("/x"), Some("/h")).unwrap(), PathBuf::from("/x/open-car/config.toml"));
        assert_eq!(config_path(Some(""), Some("/h")).unwrap(), PathBuf::from("/h/.config/open-car/config.toml"));
        assert_eq!(config_path(None, None), None);
    }

    #[test]
    fn missing_file_gives_defaults_and_round_trips() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("open-car/config.toml");
        assert_eq!(Config::load(&path).unwrap(), Config::default());
        let c = Config { profile: Some("raspberry-pi-5".into()), livi_link: true, mfi_i2c_bus: Some(1) };
        c.save(&path).unwrap();
        assert_eq!(Config::load(&path).unwrap(), c);
    }

    #[test]
    fn invalid_file_is_an_error_not_defaults() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        std::fs::write(&path, "profile = 3").unwrap();
        assert!(matches!(Config::load(&path), Err(ConfigError::Parse(..))));
    }
}
