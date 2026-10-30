// SPDX-License-Identifier: GPL-3.0-or-later
//! The Linux probe: reads sysfs, procfs and /dev under a root folder.

use super::*;
use crate::fsutil::read_trimmed;
use std::fs;
use std::path::Path;

/// Probes the computer whose file system starts at `root` (`/` for this computer).
pub fn probe(root: &Path, cmds: &dyn CommandRunner) -> HardwareReport {
    let cpuinfo = fs::read_to_string(root.join("proc/cpuinfo")).unwrap_or_default();
    HardwareReport {
        cpu_arch: std::env::consts::ARCH.to_string(),
        board_model: read_trimmed(&root.join("sys/firmware/devicetree/base/model"))
            .or_else(|| read_trimmed(&root.join("proc/device-tree/model")))
            .or_else(|| read_trimmed(&root.join("sys/class/dmi/id/product_name")))
            .filter(|s| !s.is_empty()),
        cpu_model: cpuinfo
            .lines()
            .find(|l| l.starts_with("model name") || l.starts_with("Model"))
            .and_then(|l| l.split_once(':'))
            .map(|(_, v)| v.trim().to_string()),
        cpu_cores: cpuinfo.lines().filter(|l| l.starts_with("processor")).count() as u32,
        ram_mb: fs::read_to_string(root.join("proc/meminfo")).ok().and_then(|t| parse_meminfo_mb(&t)),
        os: fs::read_to_string(root.join("etc/os-release")).ok().and_then(|t| parse_os_release(&t)),
        kernel: read_trimmed(&root.join("proc/sys/kernel/osrelease")),
        usb: probe_usb(root),
        bluetooth: names_in(&root.join("sys/class/bluetooth"))
            .into_iter()
            .filter(|n| n.starts_with("hci") && !n.contains(':'))
            .collect(),
        wifi: probe_wifi(root, cmds),
        displays: probe_displays(root),
        touchscreens: Vec::new(),
        rotary_inputs: Vec::new(),
        i2c_buses: {
            let mut buses: Vec<u32> =
                names_in(&root.join("dev")).iter().filter_map(|n| n.strip_prefix("i2c-")?.parse().ok()).collect();
            buses.sort_unstable();
            buses
        },
        audio_cards: fs::read_to_string(root.join("proc/asound/cards"))
            .map(|t| {
                t.lines()
                    .filter(|l| l.contains("]:"))
                    .filter_map(|l| l.split_once(" - ").map(|(_, n)| n.trim().to_string()))
                    .collect()
            })
            .unwrap_or_default(),
        decoders: cmds.exists("gst-inspect-1.0").then(|| {
            LIVI_DECODERS
                .iter()
                .map(|(el, hw)| Decoder {
                    element: el.to_string(),
                    hardware: *hw,
                    present: cmds.output("gst-inspect-1.0", &["--exists", el]).is_some(),
                })
                .collect()
        }),
        tools: LIVI_TOOLS
            .iter()
            .map(|(p, why)| Tool { program: p.to_string(), purpose: why.to_string(), present: cmds.exists(p) })
            .collect(),
    }
    .with_inputs(&fs::read_to_string(root.join("proc/bus/input/devices")).unwrap_or_default())
}

impl HardwareReport {
    fn with_inputs(mut self, devices: &str) -> Self {
        let (touch, rotary) = parse_input_devices(devices);
        self.touchscreens = touch;
        self.rotary_inputs = rotary;
        self
    }
}

fn names_in(dir: &Path) -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(dir)
        .map(|rd| rd.filter_map(|e| e.ok()).map(|e| e.file_name().to_string_lossy().into_owned()).collect())
        .unwrap_or_default();
    names.sort();
    names
}

fn probe_usb(root: &Path) -> UsbInfo {
    let base = root.join("sys/bus/usb/devices");
    let mut info = UsbInfo { gadget_controllers: names_in(&root.join("sys/class/udc")), ..Default::default() };
    for name in names_in(&base) {
        if name.contains(':') {
            continue; // an interface, not a device
        }
        if name.starts_with("usb") {
            info.host_controllers += 1;
            continue;
        }
        let dir = base.join(&name);
        let (Some(vendor_id), Some(product_id)) =
            (read_trimmed(&dir.join("idVendor")), read_trimmed(&dir.join("idProduct")))
        else {
            continue;
        };
        if read_trimmed(&dir.join("bDeviceClass")).as_deref() == Some("09") {
            continue; // a hub
        }
        let phone = phone_kind(&vendor_id, &product_id);
        info.devices.push(UsbDevice { vendor_id, product_id, product: read_trimmed(&dir.join("product")), phone });
    }
    info
}

fn phone_kind(vendor: &str, product: &str) -> Option<PhoneKind> {
    match vendor.to_ascii_lowercase().as_str() {
        "05ac" => Some(PhoneKind::Apple),
        "18d1"
            if matches!(product.to_ascii_lowercase().as_str(), "2d00" | "2d01" | "2d02" | "2d03" | "2d04" | "2d05") =>
        {
            Some(PhoneKind::AndroidAccessory)
        }
        _ => None,
    }
}

fn probe_wifi(root: &Path, cmds: &dyn CommandRunner) -> Vec<WifiInterface> {
    let base = root.join("sys/class/net");
    let has_iw = cmds.exists("iw");
    names_in(&base)
        .into_iter()
        .filter(|n| base.join(n).join("wireless").exists() || base.join(n).join("phy80211").exists())
        .map(|name| {
            let link = base.join(&name).join("phy80211");
            let phy = read_trimmed(&link.join("name"))
                .or_else(|| fs::read_link(&link).ok()?.file_name().map(|f| f.to_string_lossy().into_owned()));
            let ap_mode = match (&phy, has_iw) {
                (Some(p), true) => cmds.output("iw", &["phy", p, "info"]).map(|t| parse_iw_ap_mode(&t)),
                _ => None,
            };
            WifiInterface { name, phy, ap_mode }
        })
        .collect()
}

fn probe_displays(root: &Path) -> Vec<DisplayConnector> {
    let base = root.join("sys/class/drm");
    names_in(&base)
        .into_iter()
        .filter(|n| n.starts_with("card") && n.contains('-'))
        .filter_map(|name| {
            let status = read_trimmed(&base.join(&name).join("status"))?;
            let mut modes: Vec<String> = Vec::new();
            for m in fs::read_to_string(base.join(&name).join("modes")).unwrap_or_default().lines() {
                let m = m.trim().to_string();
                if !m.is_empty() && !modes.contains(&m) {
                    modes.push(m);
                }
            }
            Some(DisplayConnector { name, connected: status == "connected", modes })
        })
        .collect()
}

/// Total memory in MB from the text of `/proc/meminfo`.
pub fn parse_meminfo_mb(text: &str) -> Option<u64> {
    let line = text.lines().find(|l| l.starts_with("MemTotal:"))?;
    let kb: u64 = line.split_whitespace().nth(1)?.parse().ok()?;
    Some(kb / 1024)
}

/// `PRETTY_NAME` from the text of `/etc/os-release`.
pub fn parse_os_release(text: &str) -> Option<String> {
    let v = text.lines().find_map(|l| l.strip_prefix("PRETTY_NAME="))?;
    Some(v.trim().trim_matches('"').to_string())
}

/// Whether `iw phy <phy> info` output lists "AP" under "Supported interface modes".
pub fn parse_iw_ap_mode(text: &str) -> bool {
    let mut in_modes = false;
    for line in text.lines() {
        let t = line.trim();
        if t.starts_with("Supported interface modes") {
            in_modes = true;
        } else if in_modes {
            match t.strip_prefix("* ") {
                Some(mode) => {
                    if mode.trim() == "AP" {
                        return true;
                    }
                }
                None => in_modes = false,
            }
        }
    }
    false
}

/// Touchscreens and rotary knobs from the text of `/proc/bus/input/devices`.
///
/// A touchscreen is a device with the "direct" input property (`B: PROP=` bit 1) and absolute
/// axes. A rotary knob has `REL_DIAL` (bit 7) and no pointer axes (`REL_X` bit 0). Both bits
/// are in the lowest word, so the kernel's word size does not matter.
pub fn parse_input_devices(text: &str) -> (Vec<String>, Vec<String>) {
    let mut touch = Vec::new();
    let mut rotary = Vec::new();
    for block in text.split("\n\n") {
        let mut name = None;
        let (mut prop, mut abs, mut rel) = (0u64, false, 0u64);
        for line in block.lines() {
            if let Some(n) = line.strip_prefix("N: Name=") {
                name = Some(n.trim().trim_matches('"').to_string());
            } else if let Some(v) = line.strip_prefix("B: PROP=") {
                prop = lowest_word(v);
            } else if let Some(v) = line.strip_prefix("B: ABS=") {
                abs = v.split_whitespace().any(|w| !w.trim_start_matches('0').is_empty());
            } else if let Some(v) = line.strip_prefix("B: REL=") {
                rel = lowest_word(v);
            }
        }
        let Some(name) = name else { continue };
        if prop & 0b10 != 0 && abs {
            touch.push(name);
        } else if rel & (1 << 7) != 0 && rel & 1 == 0 {
            rotary.push(name);
        }
    }
    (touch, rotary)
}

fn lowest_word(bitmap: &str) -> u64 {
    bitmap.split_whitespace().last().and_then(|w| u64::from_str_radix(w, 16).ok()).unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    struct FakeCommands {
        present: Vec<&'static str>,
        outputs: HashMap<String, String>,
    }

    impl CommandRunner for FakeCommands {
        fn output(&self, program: &str, args: &[&str]) -> Option<String> {
            self.outputs.get(&format!("{program} {}", args.join(" "))).cloned()
        }
        fn exists(&self, program: &str) -> bool {
            self.present.contains(&program)
        }
    }

    fn write(root: &Path, rel: &str, text: &str) {
        let p = root.join(rel);
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(p, text).unwrap();
    }

    const IW: &str = "Wiphy phy0\n\tmax # scan SSIDs: 10\n\tSupported interface modes:\n\t\t * IBSS\n\t\t * managed\n\t\t * AP\n\t\t * P2P-client\n\tBand 1:\n";

    const INPUTS: &str = "I: Bus=0018 Vendor=0000 Product=0000 Version=0000\nN: Name=\"ft5x06\"\nB: PROP=2\nB: EV=b\nB: KEY=400 0 0 0 0 0\nB: ABS=2608000 3\n\nI: Bus=0003\nN: Name=\"Logitech USB Mouse\"\nB: PROP=0\nB: EV=17\nB: REL=1943\n\nI: Bus=0003\nN: Name=\"Car Knob\"\nB: PROP=0\nB: EV=7\nB: REL=80\n";

    fn fake_root() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        let r = dir.path();
        write(
            r,
            "proc/cpuinfo",
            "processor\t: 0\nprocessor\t: 1\nprocessor\t: 2\nprocessor\t: 3\nModel\t\t: Raspberry Pi 5 Model B Rev 1.0\n",
        );
        write(r, "proc/meminfo", "MemTotal:        8245168 kB\nMemFree: 1 kB\n");
        write(r, "etc/os-release", "NAME=\"Debian\"\nPRETTY_NAME=\"Debian GNU/Linux 13 (trixie)\"\n");
        write(r, "proc/sys/kernel/osrelease", "6.12.47+rpt-rpi-2712\n");
        write(r, "sys/firmware/devicetree/base/model", "Raspberry Pi 5 Model B Rev 1.0\0");
        write(r, "sys/bus/usb/devices/usb1/idVendor", "1d6b");
        write(r, "sys/bus/usb/devices/usb2/idVendor", "1d6b");
        write(r, "sys/bus/usb/devices/1-1/idVendor", "05ac");
        write(r, "sys/bus/usb/devices/1-1/idProduct", "12a8");
        write(r, "sys/bus/usb/devices/1-1/product", "iPhone");
        write(r, "sys/bus/usb/devices/1-1/serial", "SECRET-SERIAL");
        write(r, "sys/bus/usb/devices/1-1:1.0/bInterfaceClass", "06");
        write(r, "sys/bus/usb/devices/1-2/idVendor", "2109");
        write(r, "sys/bus/usb/devices/1-2/idProduct", "3431");
        write(r, "sys/bus/usb/devices/1-2/bDeviceClass", "09");
        write(r, "sys/bus/usb/devices/2-1/idVendor", "18d1");
        write(r, "sys/bus/usb/devices/2-1/idProduct", "2D01");
        write(r, "sys/class/udc/1000480000.usb/state", "not attached");
        write(r, "sys/class/bluetooth/hci0/address", "AA:BB:CC:DD:EE:FF");
        write(r, "sys/class/bluetooth/hci0:12/x", "");
        write(r, "sys/class/net/wlan0/phy80211/name", "phy0");
        write(r, "sys/class/net/eth0/address", "11:22:33:44:55:66");
        write(r, "sys/class/drm/card1-HDMI-A-1/status", "connected");
        write(r, "sys/class/drm/card1-HDMI-A-1/modes", "1280x720\n1280x720\n800x480\n");
        write(r, "sys/class/drm/card1-HDMI-A-2/status", "disconnected");
        write(r, "sys/class/drm/card1/dev", "226:1");
        write(r, "dev/i2c-1", "");
        write(r, "dev/i2c-13", "");
        write(r, "dev/i2c-x", "");
        write(
            r,
            "proc/asound/cards",
            " 0 [vc4hdmi0      ]: vc4-hdmi - vc4-hdmi-0\n                      vc4-hdmi-0\n 1 [Device         ]: USB-Audio - USB Audio Device\n",
        );
        write(r, "proc/bus/input/devices", INPUTS);
        dir
    }

    fn commands() -> FakeCommands {
        let mut outputs = HashMap::new();
        outputs.insert("iw phy phy0 info".to_string(), IW.to_string());
        outputs.insert("gst-inspect-1.0 --exists v4l2h264dec".to_string(), String::new());
        outputs.insert("gst-inspect-1.0 --exists avdec_h264".to_string(), String::new());
        FakeCommands { present: vec!["iw", "gst-inspect-1.0", "hostapd", "bluetoothctl"], outputs }
    }

    #[test]
    fn reads_a_raspberry_pi_like_tree() {
        let root = fake_root();
        let r = probe(root.path(), &commands());
        assert_eq!(r.board_model.as_deref(), Some("Raspberry Pi 5 Model B Rev 1.0"));
        assert_eq!(r.cpu_model.as_deref(), Some("Raspberry Pi 5 Model B Rev 1.0"));
        assert_eq!(r.cpu_cores, 4);
        assert_eq!(r.ram_mb, Some(8051));
        assert_eq!(r.os.as_deref(), Some("Debian GNU/Linux 13 (trixie)"));
        assert_eq!(r.kernel.as_deref(), Some("6.12.47+rpt-rpi-2712"));
        assert_eq!(r.usb.host_controllers, 2);
        assert_eq!(r.usb.gadget_controllers, vec!["1000480000.usb"]);
        assert_eq!(r.usb.devices.len(), 2, "hub and interface left out");
        assert_eq!(r.phones().len(), 2);
        assert_eq!(r.usb.devices[0].phone, Some(PhoneKind::Apple));
        assert_eq!(r.usb.devices[1].phone, Some(PhoneKind::AndroidAccessory));
        assert_eq!(r.bluetooth, vec!["hci0"]);
        assert_eq!(r.wifi, vec![WifiInterface { name: "wlan0".into(), phy: Some("phy0".into()), ap_mode: Some(true) }]);
        assert_eq!(r.displays.len(), 2);
        assert!(r.displays[0].connected);
        assert_eq!(r.displays[0].modes, vec!["1280x720", "800x480"]);
        assert_eq!(r.i2c_buses, vec![1, 13]);
        assert_eq!(r.audio_cards, vec!["vc4-hdmi-0", "USB Audio Device"]);
        assert_eq!(r.touchscreens, vec!["ft5x06"]);
        assert_eq!(r.rotary_inputs, vec!["Car Knob"]);
        let decoders = r.decoders.as_ref().unwrap();
        assert!(decoders.iter().any(|d| d.element == "v4l2h264dec" && d.present && d.hardware));
        assert!(decoders.iter().any(|d| d.element == "vah264dec" && !d.present));
        assert!(r.has_tool("hostapd") && !r.has_tool("dnsmasq"));
    }

    #[test]
    fn report_holds_no_serials_or_addresses() {
        let root = fake_root();
        let json = serde_json::to_string(&probe(root.path(), &commands())).unwrap();
        for secret in ["SECRET-SERIAL", "AA:BB:CC", "11:22:33"] {
            assert!(!json.contains(secret), "{secret} leaked");
        }
    }

    #[test]
    fn empty_root_gives_an_empty_report_not_a_crash() {
        let dir = tempfile::tempdir().unwrap();
        let none = FakeCommands { present: vec![], outputs: HashMap::new() };
        let r = probe(dir.path(), &none);
        assert_eq!(r.usb, UsbInfo::default());
        assert!(r.wifi.is_empty() && r.displays.is_empty() && r.decoders.is_none());
        assert!(r.tools.iter().all(|t| !t.present));
    }

    #[test]
    fn wifi_ap_mode_is_unknown_without_iw() {
        let root = fake_root();
        let mut c = commands();
        c.present.retain(|p| *p != "iw");
        assert_eq!(probe(root.path(), &c).wifi[0].ap_mode, None);
    }

    #[test]
    fn iw_parsing() {
        assert!(parse_iw_ap_mode(IW));
        assert!(!parse_iw_ap_mode(
            "\tSupported interface modes:\n\t\t * managed\n\t\t * monitor\n\tBand 1:\n\t\t * AP\n"
        ));
        assert!(!parse_iw_ap_mode(""));
    }

    #[test]
    fn small_parsers() {
        assert_eq!(parse_meminfo_mb("MemTotal: 2048 kB"), Some(2));
        assert_eq!(parse_meminfo_mb("nothing"), None);
        assert_eq!(parse_os_release("PRETTY_NAME=Arch Linux"), Some("Arch Linux".into()));
    }
}
