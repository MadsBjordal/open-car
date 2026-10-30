// SPDX-License-Identifier: GPL-3.0-or-later
//! Hardware profiles: one head unit, adapter or computer, as described in `compat/profiles/*.toml`.
//!
//! The rules here keep the database honest (ADR-0005): nothing is "verified" or "supported"
//! without test evidence, and CarPlay is never claimed without a named authentication route.

use serde::{Deserialize, Serialize};
use std::fmt;

/// How far a device is known to work with open-car.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CompatStatus {
    /// Tested on the real device, with evidence recorded in the profile.
    Verified,
    /// Runs on the real device, with known gaps.
    Experimental,
    /// Nobody has tested it yet.
    Untested,
    /// Cannot run open-car, directly or with an adapter.
    Unsupported,
    /// The device cannot run open-car itself; an external computer or adapter is needed.
    RequiresExternalHardware,
}

impl CompatStatus {
    /// The label people read, e.g. "Requires external hardware".
    pub fn label(self) -> &'static str {
        match self {
            Self::Verified => "Verified",
            Self::Experimental => "Experimental",
            Self::Untested => "Untested",
            Self::Unsupported => "Unsupported",
            Self::RequiresExternalHardware => "Requires external hardware",
        }
    }
}

/// Whether one projection feature (Android Auto or CarPlay) works on a device.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Support {
    /// Works, shown by test evidence.
    Yes,
    /// Works partly or unreliably.
    Experimental,
    /// Works only through an external adapter computer.
    ViaAdapter,
    /// Does not work.
    No,
    /// Not known.
    Unknown,
}

/// How open-car gets onto a device.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum InstallMethod {
    /// The device runs Linux and LIVI + open-car are installed on it.
    NativeLinux,
    /// The device runs Android and takes an APK (open-car's Android app, planned).
    NativeAndroid,
    /// A separate computer (e.g. a Raspberry Pi) runs open-car and drives the screen.
    ExternalAdapter,
    /// Nothing can be installed.
    None,
    /// Not known yet.
    Unknown,
}

/// A screen's size.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Screen {
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
    /// Diagonal in inches, when known.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub diagonal_in: Option<f32>,
}

/// One test of a device, the only thing that may make a profile "verified".
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Evidence {
    /// Day of the test, YYYY-MM-DD.
    pub date: String,
    /// Who tested (a GitHub name, never personal data).
    pub tester: String,
    /// open-car and LIVI versions used.
    pub software: String,
    /// What was tested and what happened.
    pub result: String,
    /// Link to the report (issue, photos, logs).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub link: Option<String>,
}

/// LIVI settings a profile needs, written into LIVI's `config.json` by `opencar livi configure`.
///
/// Only keys documented in LIVI's README are listed (ADR-0004).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LiviSettings {
    /// I²C bus of the MFi coprocessor (`carPlayMfiI2cBus`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mfi_i2c_bus: Option<u32>,
    /// GPIO that powers the MFi coprocessor (`carPlayMfiPowerGpio`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mfi_power_gpio: Option<u32>,
    /// Read a GPS receiver (`gpsEnabled`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gps_enabled: Option<bool>,
    /// Serial device of the GPS receiver (`gpsDevice`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gps_device: Option<String>,
    /// Baud rate of the GPS receiver (`gpsBaudRate`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gps_baud_rate: Option<u32>,
}

/// One device in the compatibility database. Unknown values are left out, never guessed.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DeviceProfile {
    /// Unique id, the file name without `.toml` (lowercase letters, digits and dashes).
    pub id: String,
    /// Maker of the device.
    pub manufacturer: String,
    /// Model name or number.
    pub model: String,
    /// Vehicles the device was fitted to, e.g. "Scion FR-S 2013-2016".
    #[serde(default)]
    pub vehicles: Vec<String>,
    /// Hardware revision.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hardware_revision: Option<String>,
    /// Operating system, e.g. "Debian 13 (Raspberry Pi OS)".
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub operating_system: Option<String>,
    /// Firmware version.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub firmware_version: Option<String>,
    /// CPU architecture, e.g. "aarch64".
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cpu_arch: Option<String>,
    /// Memory in MB.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ram_mb: Option<u32>,
    /// The screen.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub screen: Option<Screen>,
    /// USB abilities (host ports, OTG).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub usb: Option<String>,
    /// Bluetooth abilities.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bluetooth: Option<String>,
    /// Wi-Fi abilities (access-point mode matters for wireless projection).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wifi: Option<String>,
    /// Hardware video decoding.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub video_decode: Option<String>,
    /// Audio outputs and microphone.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audio: Option<String>,
    /// Has a touchscreen.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub touchscreen: Option<bool>,
    /// Buttons, knobs, steering-wheel controls.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub physical_controls: Option<String>,
    /// How open-car is installed.
    pub installation: InstallMethod,
    /// Android Auto support.
    pub android_auto: Support,
    /// CarPlay support.
    pub carplay: Support,
    /// How CarPlay authenticates, e.g. "MFi coprocessor on I2C bus 1" or "LIVI Link".
    /// Required when `carplay` is `yes` or `experimental`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub carplay_auth: Option<String>,
    /// Overall status.
    pub status: CompatStatus,
    /// Known problems.
    #[serde(default)]
    pub known_issues: Vec<String>,
    /// Tests done on the real device.
    #[serde(default)]
    pub evidence: Vec<Evidence>,
    /// Free notes, including where each fact came from.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    /// LIVI settings this device needs.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub livi: Option<LiviSettings>,
}

/// Something wrong with a profile.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Problem(pub String);

impl fmt::Display for Problem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// True when `id` is lowercase letters, digits and single dashes, starting with a letter or digit.
pub fn valid_id(id: &str) -> bool {
    !id.is_empty()
        && !id.starts_with('-')
        && !id.ends_with('-')
        && !id.contains("--")
        && id.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

impl DeviceProfile {
    /// Parses a profile from TOML text.
    pub fn from_toml(text: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(text)
    }

    /// The honesty rules a profile must follow. An empty list means the profile is valid.
    pub fn validate(&self) -> Vec<Problem> {
        let mut out = Vec::new();
        let mut bad = |s: String| out.push(Problem(s));
        if !valid_id(&self.id) {
            bad(format!("id '{}' must be lowercase letters, digits and dashes", self.id));
        }
        if self.manufacturer.trim().is_empty() || self.model.trim().is_empty() {
            bad("manufacturer and model must not be empty".into());
        }
        let tested = !self.evidence.is_empty();
        if self.status == CompatStatus::Verified && !tested {
            bad("status 'verified' needs at least one [[evidence]] entry".into());
        }
        for (name, s) in [("android_auto", self.android_auto), ("carplay", self.carplay)] {
            if s == Support::Yes && !tested {
                bad(format!("{name} = 'yes' needs at least one [[evidence]] entry"));
            }
        }
        if matches!(self.carplay, Support::Yes | Support::Experimental)
            && self.carplay_auth.as_deref().is_none_or(|a| a.trim().is_empty())
        {
            bad("carplay needs 'carplay_auth' (MFi coprocessor or LIVI Link): open-car never bypasses Apple authentication".into());
        }
        if self.status == CompatStatus::RequiresExternalHardware
            && !matches!(self.installation, InstallMethod::ExternalAdapter)
        {
            bad("status 'requires-external-hardware' needs installation = 'external-adapter'".into());
        }
        if self.status == CompatStatus::Unsupported
            && (matches!(self.android_auto, Support::Yes | Support::Experimental | Support::ViaAdapter)
                || matches!(self.carplay, Support::Yes | Support::Experimental | Support::ViaAdapter))
        {
            bad("an 'unsupported' device cannot list Android Auto or CarPlay as working".into());
        }
        if matches!(self.status, CompatStatus::Untested)
            && (self.android_auto == Support::Yes || self.carplay == Support::Yes)
        {
            bad("an 'untested' device cannot list Android Auto or CarPlay as 'yes'".into());
        }
        for e in &self.evidence {
            if !is_date(&e.date) {
                bad(format!("evidence date '{}' must be YYYY-MM-DD", e.date));
            }
            if e.tester.trim().is_empty() || e.result.trim().is_empty() {
                bad("evidence needs a tester and a result".into());
            }
        }
        out
    }

    /// Text the database search looks through: id, maker, model, vehicles.
    pub fn search_text(&self) -> String {
        let mut s = format!("{} {} {}", self.id, self.manufacturer, self.model);
        for v in &self.vehicles {
            s.push(' ');
            s.push_str(v);
        }
        s.to_lowercase()
    }
}

fn is_date(s: &str) -> bool {
    let b = s.as_bytes();
    b.len() == 10
        && b[4] == b'-'
        && b[7] == b'-'
        && b.iter().enumerate().all(|(i, c)| i == 4 || i == 7 || c.is_ascii_digit())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> DeviceProfile {
        DeviceProfile::from_toml(
            r#"
            id = "test-unit"
            manufacturer = "Acme"
            model = "HU-1"
            installation = "native-linux"
            android_auto = "unknown"
            carplay = "unknown"
            status = "untested"
            "#,
        )
        .unwrap()
    }

    fn evidence() -> Evidence {
        Evidence {
            date: "2026-10-30".into(),
            tester: "someone".into(),
            software: "open-car 0.1.0, LIVI 2026-10-29".into(),
            result: "wired Android Auto ran for an hour".into(),
            link: None,
        }
    }

    #[test]
    fn a_plain_untested_profile_is_valid() {
        assert!(base().validate().is_empty());
    }

    #[test]
    fn verified_needs_evidence() {
        let mut p = base();
        p.status = CompatStatus::Verified;
        assert_eq!(p.validate().len(), 1);
        p.evidence.push(evidence());
        assert!(p.validate().is_empty());
    }

    #[test]
    fn carplay_needs_an_authentication_route() {
        let mut p = base();
        p.status = CompatStatus::Experimental;
        p.carplay = Support::Experimental;
        assert!(p.validate()[0].0.contains("carplay_auth"));
        p.carplay_auth = Some("LIVI Link".into());
        assert!(p.validate().is_empty());
    }

    #[test]
    fn support_yes_needs_evidence_and_untested_cannot_say_yes() {
        let mut p = base();
        p.android_auto = Support::Yes;
        let problems = p.validate();
        assert!(problems.iter().any(|x| x.0.contains("evidence")));
        assert!(problems.iter().any(|x| x.0.contains("untested")));
    }

    #[test]
    fn external_hardware_status_needs_adapter_installation() {
        let mut p = base();
        p.status = CompatStatus::RequiresExternalHardware;
        assert_eq!(p.validate().len(), 1);
        p.installation = InstallMethod::ExternalAdapter;
        assert!(p.validate().is_empty());
    }

    #[test]
    fn unsupported_cannot_list_working_features() {
        let mut p = base();
        p.status = CompatStatus::Unsupported;
        p.installation = InstallMethod::None;
        p.android_auto = Support::ViaAdapter;
        assert_eq!(p.validate().len(), 1);
    }

    #[test]
    fn rejects_bad_ids_dates_and_unknown_fields() {
        let mut p = base();
        p.id = "Bad_Id".into();
        let mut e = evidence();
        e.date = "30.10.2026".into();
        p.evidence.push(e);
        assert_eq!(p.validate().len(), 2);
        let err = DeviceProfile::from_toml(
            "id='a'\nmanufacturer='x'\nmodel='y'\ninstallation='none'\nandroid_auto='no'\ncarplay='no'\nstatus='unsupported'\ncolour='red'",
        );
        assert!(err.is_err());
    }

    #[test]
    fn ids() {
        assert!(valid_id("raspberry-pi-5"));
        for bad in ["", "-a", "a-", "a--b", "A", "a_b", "a b"] {
            assert!(!valid_id(bad), "{bad}");
        }
    }
}
