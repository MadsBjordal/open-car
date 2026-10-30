// SPDX-License-Identifier: GPL-3.0-or-later
//! LIVI, the projection engine open-car builds on (ADR-0002): where it is installed, and
//! writing a hardware profile's settings into its `config.json` without touching anything else.
//!
//! Paths and keys follow LIVI's own code and README at commit e7b20b8 (2026-10-29):
//! `livi-core/src/paths.rs` (config in `$XDG_CONFIG_HOME/LIVI`, else `~/.config/LIVI`) and the
//! installers (`~/LIVI/LIVI.AppImage`, `livi-kiosk.service`).

use crate::fsutil::write_atomic;
use crate::profile::LiviSettings;
use serde::Serialize;
use serde_json::{Map, Value};
use std::path::{Path, PathBuf};

/// Where LIVI keeps its settings: `$XDG_CONFIG_HOME/LIVI/config.json`, else `$HOME/.config/LIVI/config.json`.
pub fn config_path(xdg_config_home: Option<&str>, home: Option<&str>) -> Option<PathBuf> {
    let base = match (xdg_config_home.filter(|s| !s.is_empty()), home.filter(|s| !s.is_empty())) {
        (Some(x), _) => PathBuf::from(x),
        (None, Some(h)) => Path::new(h).join(".config"),
        (None, None) => return None,
    };
    Some(base.join("LIVI/config.json"))
}

/// What is installed of LIVI on this computer.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct Install {
    /// The AppImage LIVI's installer puts in `~/LIVI/LIVI.AppImage`.
    pub appimage: Option<PathBuf>,
    /// LIVI's settings file exists (LIVI has run at least once).
    pub config: Option<PathBuf>,
    /// The headless installer's start-at-boot service `livi-kiosk.service` is installed.
    pub kiosk_service: bool,
    /// The desktop installer's autostart entry is installed.
    pub desktop_autostart: bool,
}

/// Looks for LIVI's files. `root` is `/` on a real system; `home` and `xdg_config_home` are the user's.
pub fn find_install(root: &Path, home: Option<&str>, xdg_config_home: Option<&str>) -> Install {
    let in_root = |p: &Path| root.join(p.strip_prefix("/").unwrap_or(p));
    let appimage = home.map(|h| in_root(&Path::new(h).join("LIVI/LIVI.AppImage"))).filter(|p| p.is_file());
    let config = config_path(xdg_config_home, home).map(|p| in_root(&p)).filter(|p| p.is_file());
    let autostart_dir =
        config_path(xdg_config_home, home).and_then(|p| p.parent()?.parent().map(|d| in_root(&d.join("autostart"))));
    let desktop_autostart = autostart_dir
        .and_then(|d| std::fs::read_dir(d).ok())
        .map(|rd| {
            rd.filter_map(|e| e.ok()).any(|e| {
                let n = e.file_name().to_string_lossy().to_lowercase();
                n.contains("livi") && n.ends_with(".desktop")
            })
        })
        .unwrap_or(false);
    Install {
        appimage,
        config,
        kiosk_service: root.join("etc/systemd/system/livi-kiosk.service").is_file(),
        desktop_autostart,
    }
}

/// One setting changed in LIVI's config.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Change {
    /// LIVI's key, e.g. `carPlayMfiI2cBus`.
    pub key: String,
    /// The value before (`null` when it was not set).
    pub before: Value,
    /// The new value.
    pub after: Value,
}

/// Errors changing LIVI's config.
#[derive(Debug, thiserror::Error)]
pub enum LiviError {
    /// The file could not be read or written.
    #[error("{0}: {1}")]
    Io(PathBuf, std::io::Error),
    /// The file is not a JSON object; it is left as it is.
    #[error("{0} is not a JSON object, so it was left untouched: {1}")]
    NotAnObject(PathBuf, String),
}

/// LIVI's config keys for a profile's settings, only those the profile sets.
pub fn settings_to_keys(s: &LiviSettings) -> Vec<(&'static str, Value)> {
    let mut out = Vec::new();
    if let Some(v) = s.mfi_i2c_bus {
        out.push(("carPlayMfiI2cBus", Value::from(v)));
    }
    if let Some(v) = s.mfi_power_gpio {
        out.push(("carPlayMfiPowerGpio", Value::from(v)));
    }
    if let Some(v) = s.gps_enabled {
        out.push(("gpsEnabled", Value::from(v)));
    }
    if let Some(v) = &s.gps_device {
        out.push(("gpsDevice", Value::from(v.clone())));
    }
    if let Some(v) = s.gps_baud_rate {
        out.push(("gpsBaudRate", Value::from(v)));
    }
    out
}

/// The changes `apply_settings` would make, without writing anything.
pub fn plan_settings(path: &Path, settings: &LiviSettings) -> Result<Vec<Change>, LiviError> {
    let current = read_object(path)?;
    Ok(settings_to_keys(settings)
        .into_iter()
        .filter_map(|(k, v)| {
            let before = current.get(k).cloned().unwrap_or(Value::Null);
            (before != v).then(|| Change { key: k.to_string(), before, after: v })
        })
        .collect())
}

/// Writes a profile's settings into LIVI's config, keeping every other key as it was.
///
/// The old file is kept as `config.json.bak` and the write is safe against power loss. LIVI
/// should be stopped first: a running LIVI may write its own copy back. Returns what changed.
pub fn apply_settings(path: &Path, settings: &LiviSettings) -> Result<Vec<Change>, LiviError> {
    let changes = plan_settings(path, settings)?;
    if changes.is_empty() {
        return Ok(changes);
    }
    let mut obj = read_object(path)?;
    for c in &changes {
        obj.insert(c.key.clone(), c.after.clone());
    }
    let mut text = serde_json::to_string_pretty(&Value::Object(obj)).expect("JSON always serialises");
    text.push('\n');
    write_atomic(path, text.as_bytes(), true).map_err(|e| LiviError::Io(path.into(), e))?;
    Ok(changes)
}

fn read_object(path: &Path) -> Result<Map<String, Value>, LiviError> {
    let text = match std::fs::read_to_string(path) {
        Ok(t) => t,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Map::new()),
        Err(e) => return Err(LiviError::Io(path.into(), e)),
    };
    match serde_json::from_str::<Value>(&text) {
        Ok(Value::Object(m)) => Ok(m),
        Ok(other) => Err(LiviError::NotAnObject(path.into(), format!("found {}", kind(&other)))),
        Err(e) => Err(LiviError::NotAnObject(path.into(), e.to_string())),
    }
}

fn kind(v: &Value) -> &'static str {
    match v {
        Value::Null => "null",
        Value::Bool(_) => "a boolean",
        Value::Number(_) => "a number",
        Value::String(_) => "a string",
        Value::Array(_) => "an array",
        Value::Object(_) => "an object",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fsutil::backup_path;
    use std::fs;

    fn mfi() -> LiviSettings {
        LiviSettings { mfi_i2c_bus: Some(1), mfi_power_gpio: Some(21), ..Default::default() }
    }

    #[test]
    fn config_path_matches_livi() {
        assert_eq!(config_path(None, Some("/home/u")).unwrap(), PathBuf::from("/home/u/.config/LIVI/config.json"));
        assert_eq!(config_path(Some("/cfg"), Some("/home/u")).unwrap(), PathBuf::from("/cfg/LIVI/config.json"));
    }

    #[test]
    fn keeps_other_keys_and_backs_up() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.json");
        fs::write(&path, r#"{"carName":"FR-S","carPlayMfiI2cBus":2,"carPlayMfiPowerGpio":21}"#).unwrap();
        let changes = apply_settings(&path, &mfi()).unwrap();
        assert_eq!(
            changes,
            vec![Change { key: "carPlayMfiI2cBus".into(), before: Value::from(2), after: Value::from(1) }]
        );
        let v: Value = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(v["carName"], "FR-S");
        assert_eq!(v["carPlayMfiI2cBus"], 1);
        assert!(backup_path(&path).exists());
        assert!(apply_settings(&path, &mfi()).unwrap().is_empty(), "second run changes nothing");
    }

    #[test]
    fn creates_a_missing_config_and_plans_without_writing() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("LIVI/config.json");
        assert_eq!(plan_settings(&path, &mfi()).unwrap().len(), 2);
        assert!(!path.exists());
        apply_settings(&path, &mfi()).unwrap();
        assert!(path.exists());
    }

    #[test]
    fn refuses_a_file_that_is_not_an_object() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.json");
        fs::write(&path, "[1,2]").unwrap();
        assert!(matches!(apply_settings(&path, &mfi()), Err(LiviError::NotAnObject(..))));
        assert_eq!(fs::read_to_string(&path).unwrap(), "[1,2]");
        fs::write(&path, "{broken").unwrap();
        assert!(matches!(apply_settings(&path, &mfi()), Err(LiviError::NotAnObject(..))));
    }

    #[test]
    fn finds_the_installers_files() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        assert_eq!(find_install(root, Some("/home/pi"), None), Install::default());
        for f in [
            "home/pi/LIVI/LIVI.AppImage",
            "home/pi/.config/LIVI/config.json",
            "home/pi/.config/autostart/LIVI.desktop",
            "etc/systemd/system/livi-kiosk.service",
        ] {
            let p = root.join(f);
            fs::create_dir_all(p.parent().unwrap()).unwrap();
            fs::write(p, "").unwrap();
        }
        let i = find_install(root, Some("/home/pi"), None);
        assert!(i.appimage.is_some() && i.config.is_some() && i.kiosk_service && i.desktop_autostart);
    }
}
