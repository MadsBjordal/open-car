// SPDX-License-Identifier: GPL-3.0-or-later
//! Diagnostics: one JSON file describing this computer for a bug or compatibility report.
//!
//! It is written locally and never sent anywhere (MASTER_SECURITY.md). It holds no MAC
//! addresses, serial numbers, host names or phone data, and the user's home folder is shown as `~`.

use crate::livi::Install;
use crate::probe::HardwareReport;
use crate::readiness::Check;
use serde::Serialize;
use std::path::{Path, PathBuf};

/// Everything in a diagnostics file.
#[derive(Debug, Clone, Serialize)]
pub struct Diagnostics {
    /// open-car version.
    pub opencar_version: String,
    /// Hardware profile chosen in open-car's settings.
    pub profile: Option<String>,
    /// What the probe found.
    pub hardware: HardwareReport,
    /// The readiness checks.
    pub readiness: Vec<Check>,
    /// What is installed of LIVI.
    pub livi: Install,
}

impl Diagnostics {
    /// The diagnostics as pretty JSON, with the home folder replaced by `~`.
    pub fn to_redacted_json(&self, home: Option<&str>) -> String {
        let mut d = self.clone();
        if let Some(h) = home.filter(|h| h.len() > 1) {
            for p in [&mut d.livi.appimage, &mut d.livi.config].into_iter().flatten() {
                *p = redact_home(p, h);
            }
        }
        serde_json::to_string_pretty(&d).expect("diagnostics always serialise")
    }
}

fn redact_home(p: &Path, home: &str) -> PathBuf {
    match p.to_str().and_then(|s| s.find(home).map(|i| (s, i))) {
        Some((s, i)) => PathBuf::from(format!("~{}", &s[i + home.len()..])),
        None => p.to_path_buf(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn home_folder_is_hidden() {
        let d = Diagnostics {
            opencar_version: "0.1.0".into(),
            profile: None,
            hardware: HardwareReport::default(),
            readiness: vec![],
            livi: Install { appimage: Some("/home/mads/LIVI/LIVI.AppImage".into()), ..Default::default() },
        };
        let json = d.to_redacted_json(Some("/home/mads"));
        assert!(json.contains("~/LIVI/LIVI.AppImage"));
        assert!(!json.contains("mads"));
    }
}
