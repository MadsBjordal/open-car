// SPDX-License-Identifier: GPL-3.0-or-later
//! Readiness: from a [`HardwareReport`], what this computer can do with LIVI, and what is missing.
//!
//! The rules follow LIVI's documented requirements (README, `scripts/install/packages.txt`,
//! evaluated at LIVI commit e7b20b8). "Ready" means the requirements are present, not that a
//! phone has been tested on this computer: only a real test can show that (ADR-0005).

use crate::probe::HardwareReport;
use serde::Serialize;

/// A feature a head unit offers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Feature {
    /// A screen is attached.
    Display,
    /// Video decoding on the GPU or a video block.
    HardwareVideoDecode,
    /// A touchscreen.
    Touch,
    /// Android Auto over a USB cable.
    WiredAndroidAuto,
    /// Android Auto over Bluetooth + Wi-Fi.
    WirelessAndroidAuto,
    /// CarPlay over a USB cable.
    WiredCarPlay,
    /// CarPlay over Bluetooth + Wi-Fi.
    WirelessCarPlay,
}

impl Feature {
    /// The name people read.
    pub fn label(self) -> &'static str {
        match self {
            Self::Display => "Display",
            Self::HardwareVideoDecode => "Hardware video decoding",
            Self::Touch => "Touchscreen",
            Self::WiredAndroidAuto => "Wired Android Auto",
            Self::WirelessAndroidAuto => "Wireless Android Auto",
            Self::WiredCarPlay => "Wired CarPlay",
            Self::WirelessCarPlay => "Wireless CarPlay",
        }
    }
}

/// How ready a feature is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Level {
    /// Everything it needs is present.
    Ready,
    /// Works, with a limitation (e.g. software video decoding).
    Limited,
    /// Could not be decided from what the probe can see.
    Unknown,
    /// Something it needs is missing.
    Missing,
}

/// The verdict for one feature, with the reasons in plain words.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Check {
    /// The feature.
    pub feature: Feature,
    /// How ready it is.
    pub level: Level,
    /// Why, and what to do about it.
    pub reasons: Vec<String>,
}

/// What the probe cannot see for itself, from open-car's settings.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Options {
    /// A LIVI Link box is connected (its own radios and CarPlay authentication).
    pub livi_link: bool,
    /// I²C bus an MFi coprocessor is said to be on.
    pub mfi_i2c_bus: Option<u32>,
}

/// Checks every [`Feature`] against the report.
pub fn assess(r: &HardwareReport, opts: Options) -> Vec<Check> {
    let usb = usb_check(r);
    let radios = wireless_check(r, opts);
    let mfi = carplay_auth_check(r, opts);
    let mut avahi = Check { feature: Feature::WirelessCarPlay, level: Level::Ready, reasons: vec![] };
    if !opts.livi_link {
        for p in ["avahi-daemon", "avahi-publish-service"] {
            if !r.has_tool(p) {
                avahi.level = Level::Missing;
                avahi
                    .reasons
                    .push(format!("'{p}' is not installed (CarPlay discovery over Wi-Fi); LIVI's installer adds it"));
            }
        }
    }
    vec![
        display_check(r),
        decode_check(r),
        touch_check(r),
        combine(Feature::WiredAndroidAuto, &[&usb]),
        combine(Feature::WirelessAndroidAuto, &[&radios]),
        combine(Feature::WiredCarPlay, &[&usb, &mfi]),
        combine(Feature::WirelessCarPlay, &[&radios, &mfi, &avahi]),
    ]
}

/// Joins partial checks: the worst level wins and every reason is kept.
fn combine(feature: Feature, parts: &[&Check]) -> Check {
    let level = parts.iter().map(|c| c.level).max().unwrap_or(Level::Ready);
    let reasons = parts.iter().flat_map(|c| c.reasons.iter().cloned()).collect();
    Check { feature, level, reasons }
}

fn display_check(r: &HardwareReport) -> Check {
    let connected: Vec<_> = r.displays.iter().filter(|d| d.connected).collect();
    let (level, reasons) = if let Some(d) = connected.first() {
        let mode = d.modes.first().map(|m| format!(", {m}")).unwrap_or_default();
        (Level::Ready, vec![format!("screen on {}{mode}", d.name)])
    } else if r.displays.is_empty() {
        (Level::Unknown, vec!["no display connectors visible (no DRM driver, or not Linux)".into()])
    } else {
        (Level::Missing, vec!["no screen is connected".into()])
    };
    Check { feature: Feature::Display, level, reasons }
}

fn decode_check(r: &HardwareReport) -> Check {
    let (level, reasons) = match &r.decoders {
        None => {
            (Level::Unknown, vec!["GStreamer's gst-inspect-1.0 is not installed, so decoders cannot be listed".into()])
        }
        Some(d) => {
            let hw: Vec<_> = d.iter().filter(|x| x.hardware && x.present).map(|x| x.element.clone()).collect();
            let sw = d.iter().any(|x| !x.hardware && x.present);
            if !hw.is_empty() {
                (Level::Ready, vec![format!("hardware decoders: {}", hw.join(", "))])
            } else if sw {
                (Level::Limited, vec!["software decoding only: works, but uses the CPU and adds heat and delay".into()])
            } else {
                (
                    Level::Missing,
                    vec!["no H.264/H.265 decoder found (LIVI's AppImage brings its own software decoder)".into()],
                )
            }
        }
    };
    Check { feature: Feature::HardwareVideoDecode, level, reasons }
}

fn touch_check(r: &HardwareReport) -> Check {
    if !r.touchscreens.is_empty() {
        Check {
            feature: Feature::Touch,
            level: Level::Ready,
            reasons: vec![format!("touchscreen: {}", r.touchscreens.join(", "))],
        }
    } else {
        let mut reasons = vec!["no touchscreen found: use a knob, D-pad or keys instead".to_string()];
        if !r.rotary_inputs.is_empty() {
            reasons.push(format!("rotary controller: {}", r.rotary_inputs.join(", ")));
        }
        Check { feature: Feature::Touch, level: Level::Limited, reasons }
    }
}

fn usb_check(r: &HardwareReport) -> Check {
    let mut c = Check { feature: Feature::WiredAndroidAuto, level: Level::Ready, reasons: vec![] };
    if r.usb.host_controllers == 0 {
        c.level = Level::Missing;
        c.reasons.push("no USB host port found: a wired phone needs one (OTG device mode is not needed)".into());
    } else {
        c.reasons.push(format!("{} USB host controller(s)", r.usb.host_controllers));
    }
    c
}

fn wireless_check(r: &HardwareReport, opts: Options) -> Check {
    let mut c = Check { feature: Feature::WirelessAndroidAuto, level: Level::Ready, reasons: vec![] };
    if opts.livi_link {
        c.reasons.push("radios come from the LIVI Link".into());
        return c;
    }
    let worse = |c: &mut Check, l: Level, why: String| {
        c.level = c.level.max(l);
        c.reasons.push(why);
    };
    if r.bluetooth.is_empty() {
        worse(&mut c, Level::Missing, "no Bluetooth adapter: needed for pairing and handover".into());
    }
    if r.wifi.is_empty() {
        worse(&mut c, Level::Missing, "no Wi-Fi interface: the phone joins LIVI's own access point".into());
    } else if r.wifi.iter().any(|w| w.ap_mode == Some(true)) {
        let w = r.wifi.iter().find(|w| w.ap_mode == Some(true)).map(|w| w.name.as_str()).unwrap_or_default();
        c.reasons.push(format!("{w} can run an access point"));
    } else if r.wifi.iter().all(|w| w.ap_mode == Some(false)) {
        worse(&mut c, Level::Missing, "no Wi-Fi interface supports access-point (AP) mode".into());
    } else {
        worse(&mut c, Level::Unknown, "access-point support could not be read ('iw' missing)".into());
    }
    for p in ["bluetoothctl", "hostapd", "dnsmasq", "iw", "rfkill"] {
        if !r.has_tool(p) {
            worse(&mut c, Level::Missing, format!("'{p}' is not installed; LIVI's installer adds it"));
        }
    }
    c
}

fn carplay_auth_check(r: &HardwareReport, opts: Options) -> Check {
    let mut c = Check { feature: Feature::WiredCarPlay, level: Level::Missing, reasons: vec![] };
    if opts.livi_link {
        c.level = Level::Ready;
        c.reasons.push("authentication through the LIVI Link".into());
    } else if let Some(bus) = opts.mfi_i2c_bus {
        if r.i2c_buses.contains(&bus) {
            c.level = Level::Unknown;
            c.reasons.push(format!(
                "I2C bus {bus} exists; open-car does not talk to the bus, so whether the MFi chip answers shows only when LIVI starts a CarPlay session"
            ));
        } else {
            c.reasons
                .push(format!("MFi chip set on I2C bus {bus}, but /dev/i2c-{bus} does not exist (I2C not enabled?)"));
        }
    } else {
        c.reasons.push(
            "CarPlay needs Apple MFi authentication: an MFi coprocessor on I2C, or a LIVI Link. open-car never bypasses it".into(),
        );
    }
    c
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::probe::*;

    fn tool(p: &str, present: bool) -> Tool {
        Tool { program: p.into(), purpose: String::new(), present }
    }

    fn pi_like() -> HardwareReport {
        HardwareReport {
            usb: UsbInfo { host_controllers: 2, ..Default::default() },
            bluetooth: vec!["hci0".into()],
            wifi: vec![WifiInterface { name: "wlan0".into(), phy: Some("phy0".into()), ap_mode: Some(true) }],
            displays: vec![DisplayConnector {
                name: "card1-HDMI-A-1".into(),
                connected: true,
                modes: vec!["1280x720".into()],
            }],
            touchscreens: vec!["ft5x06".into()],
            i2c_buses: vec![1],
            decoders: Some(vec![Decoder { element: "v4l2h265dec".into(), hardware: true, present: true }]),
            tools: LIVI_TOOLS.iter().map(|(p, _)| tool(p, true)).collect(),
            ..Default::default()
        }
    }

    fn level(checks: &[Check], f: Feature) -> Level {
        checks.iter().find(|c| c.feature == f).unwrap().level
    }

    #[test]
    fn a_full_box_is_ready_for_android_auto_but_not_carplay_without_auth() {
        let c = assess(&pi_like(), Options::default());
        for f in [
            Feature::Display,
            Feature::HardwareVideoDecode,
            Feature::Touch,
            Feature::WiredAndroidAuto,
            Feature::WirelessAndroidAuto,
        ] {
            assert_eq!(level(&c, f), Level::Ready, "{f:?}");
        }
        assert_eq!(level(&c, Feature::WiredCarPlay), Level::Missing);
        assert_eq!(level(&c, Feature::WirelessCarPlay), Level::Missing);
        let why = &c.iter().find(|x| x.feature == Feature::WiredCarPlay).unwrap().reasons;
        assert!(why.iter().any(|r| r.contains("never bypasses")));
    }

    #[test]
    fn mfi_on_an_existing_bus_is_unknown_not_ready() {
        let c = assess(&pi_like(), Options { mfi_i2c_bus: Some(1), livi_link: false });
        assert_eq!(level(&c, Feature::WiredCarPlay), Level::Unknown);
        let c = assess(&pi_like(), Options { mfi_i2c_bus: Some(3), livi_link: false });
        assert_eq!(level(&c, Feature::WiredCarPlay), Level::Missing);
    }

    #[test]
    fn livi_link_brings_radios_and_authentication() {
        let mut r = pi_like();
        r.bluetooth.clear();
        r.wifi.clear();
        r.tools.clear();
        let c = assess(&r, Options { livi_link: true, mfi_i2c_bus: None });
        assert_eq!(level(&c, Feature::WirelessAndroidAuto), Level::Ready);
        assert_eq!(level(&c, Feature::WiredCarPlay), Level::Ready);
        assert_eq!(level(&c, Feature::WirelessCarPlay), Level::Ready);
    }

    #[test]
    fn wireless_needs_ap_mode_and_tools() {
        let mut r = pi_like();
        r.wifi[0].ap_mode = None;
        assert_eq!(level(&assess(&r, Options::default()), Feature::WirelessAndroidAuto), Level::Unknown);
        r.wifi[0].ap_mode = Some(false);
        assert_eq!(level(&assess(&r, Options::default()), Feature::WirelessAndroidAuto), Level::Missing);
        let mut r = pi_like();
        r.tools.retain(|t| t.program != "hostapd");
        let c = assess(&r, Options::default());
        assert_eq!(level(&c, Feature::WirelessAndroidAuto), Level::Missing);
        assert!(c[4].reasons.iter().any(|x| x.contains("hostapd")));
    }

    #[test]
    fn no_usb_host_means_no_wired_projection() {
        let mut r = pi_like();
        r.usb.host_controllers = 0;
        assert_eq!(level(&assess(&r, Options::default()), Feature::WiredAndroidAuto), Level::Missing);
    }

    #[test]
    fn decoding_and_touch_fall_back() {
        let mut r = pi_like();
        r.decoders = Some(vec![Decoder { element: "avdec_h264".into(), hardware: false, present: true }]);
        r.touchscreens.clear();
        r.rotary_inputs = vec!["Knob".into()];
        let c = assess(&r, Options::default());
        assert_eq!(level(&c, Feature::HardwareVideoDecode), Level::Limited);
        assert_eq!(level(&c, Feature::Touch), Level::Limited);
        r.decoders = None;
        r.displays.clear();
        let c = assess(&r, Options::default());
        assert_eq!(level(&c, Feature::HardwareVideoDecode), Level::Unknown);
        assert_eq!(level(&c, Feature::Display), Level::Unknown);
    }
}
