// SPDX-License-Identifier: GPL-3.0-or-later
//! Hardware probing: what this computer has that phone projection needs.
//!
//! This is the hardware abstraction layer's read side (ADR-0003). Everything is read from files
//! under a root folder (`/` on a real system, a fake tree in tests) and from a few commands
//! behind [`CommandRunner`]. Probing only reads: it never writes to a device, never talks to
//! the I²C bus and never touches the vehicle.

mod linux;

pub use linux::{parse_input_devices, parse_iw_ap_mode, parse_meminfo_mb, parse_os_release, probe};

use serde::Serialize;
use std::path::Path;
use std::process::{Command, Stdio};

/// Runs outside programs for the probe; replaced by a stand-in in tests.
pub trait CommandRunner {
    /// Runs `program args…` and gives its standard output when it exits successfully.
    fn output(&self, program: &str, args: &[&str]) -> Option<String>;
    /// True when `program` is found on the `PATH`.
    fn exists(&self, program: &str) -> bool;
}

/// The real [`CommandRunner`]: runs programs on this computer.
pub struct SystemCommands;

impl CommandRunner for SystemCommands {
    fn output(&self, program: &str, args: &[&str]) -> Option<String> {
        let out = Command::new(program).args(args).stdin(Stdio::null()).stderr(Stdio::null()).output().ok()?;
        out.status.success().then(|| String::from_utf8_lossy(&out.stdout).into_owned())
    }

    fn exists(&self, program: &str) -> bool {
        std::env::var_os("PATH")
            .map(|p| std::env::split_paths(&p).any(|d| is_executable(&d.join(program))))
            .unwrap_or(false)
    }
}

fn is_executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    path.metadata().map(|m| m.is_file() && m.permissions().mode() & 0o111 != 0).unwrap_or(false)
}

/// A USB device seen on the bus. Serial numbers are never read (privacy).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct UsbDevice {
    /// Vendor id, 4 hex digits.
    pub vendor_id: String,
    /// Product id, 4 hex digits.
    pub product_id: String,
    /// Product name the device reports.
    pub product: Option<String>,
    /// What kind of phone this is, when recognisable.
    pub phone: Option<PhoneKind>,
}

/// A phone recognised on USB by its ids.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum PhoneKind {
    /// An Apple device (vendor 05ac): an iPhone for CarPlay.
    Apple,
    /// A phone already switched to Android Open Accessory mode (18d1:2d00–2d05), as Android Auto does.
    AndroidAccessory,
}

/// USB as seen by Linux.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct UsbInfo {
    /// Number of USB host controllers (root hubs). Wired projection needs at least one.
    pub host_controllers: u32,
    /// USB device-mode (OTG gadget) controllers. LIVI does not need these for wired sessions.
    pub gadget_controllers: Vec<String>,
    /// Devices plugged in (hubs left out).
    pub devices: Vec<UsbDevice>,
}

/// A Wi-Fi interface.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct WifiInterface {
    /// Interface name, e.g. `wlan0`.
    pub name: String,
    /// Its radio, e.g. `phy0`.
    pub phy: Option<String>,
    /// Can run an access point (wireless projection needs one). `None` = could not be read (no `iw`).
    pub ap_mode: Option<bool>,
}

/// A display connector (DRM).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DisplayConnector {
    /// Connector name, e.g. `card1-HDMI-A-1`.
    pub name: String,
    /// A screen is attached.
    pub connected: bool,
    /// Modes the screen offers, best first, e.g. `1280x720`.
    pub modes: Vec<String>,
}

/// One GStreamer video decoder LIVI can use, and whether this computer has it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Decoder {
    /// GStreamer element name, e.g. `v4l2h264dec`.
    pub element: String,
    /// Runs on the GPU or a video block rather than the CPU.
    pub hardware: bool,
    /// Installed.
    pub present: bool,
}

/// A program LIVI needs (from LIVI's `scripts/install/packages.txt`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Tool {
    /// Program name, e.g. `hostapd`.
    pub program: String,
    /// What LIVI uses it for.
    pub purpose: String,
    /// Found on the `PATH`.
    pub present: bool,
}

/// Everything the probe found. Holds no MAC addresses, serial numbers, host names or user names.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct HardwareReport {
    /// CPU architecture, e.g. `aarch64`.
    pub cpu_arch: String,
    /// Board model from the device tree, e.g. "Raspberry Pi 5 Model B Rev 1.0".
    pub board_model: Option<String>,
    /// CPU model name from `/proc/cpuinfo`.
    pub cpu_model: Option<String>,
    /// Number of CPU cores.
    pub cpu_cores: u32,
    /// Memory in MB.
    pub ram_mb: Option<u64>,
    /// Operating system, e.g. "Debian GNU/Linux 13 (trixie)".
    pub os: Option<String>,
    /// Kernel release.
    pub kernel: Option<String>,
    /// USB.
    pub usb: UsbInfo,
    /// Bluetooth adapters, e.g. `hci0`.
    pub bluetooth: Vec<String>,
    /// Wi-Fi interfaces.
    pub wifi: Vec<WifiInterface>,
    /// Display connectors.
    pub displays: Vec<DisplayConnector>,
    /// Input devices with multi-touch (touchscreens).
    pub touchscreens: Vec<String>,
    /// Input devices with a rotary knob (a dial without a pointer), e.g. a car controller.
    pub rotary_inputs: Vec<String>,
    /// I²C bus numbers (`/dev/i2c-N`). An MFi coprocessor sits on one of these.
    pub i2c_buses: Vec<u32>,
    /// Sound cards.
    pub audio_cards: Vec<String>,
    /// Video decoders; `None` when GStreamer's `gst-inspect-1.0` is not installed.
    pub decoders: Option<Vec<Decoder>>,
    /// Programs LIVI needs.
    pub tools: Vec<Tool>,
}

impl HardwareReport {
    /// Phones currently plugged in by USB.
    pub fn phones(&self) -> Vec<&UsbDevice> {
        self.usb.devices.iter().filter(|d| d.phone.is_some()).collect()
    }

    /// Whether a tool LIVI needs is installed; `false` when it is not in the list.
    pub fn has_tool(&self, program: &str) -> bool {
        self.tools.iter().any(|t| t.program == program && t.present)
    }
}

/// The decoders LIVI looks for (from LIVI's video code), and whether each is hardware.
pub const LIVI_DECODERS: &[(&str, bool)] = &[
    ("v4l2h264dec", true),
    ("v4l2h265dec", true),
    ("v4l2slh264dec", true),
    ("v4l2slh265dec", true),
    ("vah264dec", true),
    ("vah265dec", true),
    ("nvh264dec", true),
    ("nvh265dec", true),
    ("avdec_h264", false),
    ("avdec_h265", false),
];

/// The programs LIVI needs on every host (the `cmd:` probes of LIVI's `packages.txt`, core section).
pub const LIVI_TOOLS: &[(&str, &str)] = &[
    ("bluetoothctl", "Bluetooth pairing for wireless Android Auto and CarPlay"),
    ("hostapd", "Wi-Fi access point for wireless Android Auto and CarPlay"),
    ("dnsmasq", "DHCP for the wireless access point"),
    ("iw", "Wi-Fi interface control"),
    ("rfkill", "Wi-Fi and Bluetooth radio control"),
    ("pactl", "Reading and setting the system volume"),
    ("avahi-daemon", "CarPlay discovery over Wi-Fi"),
    ("avahi-publish-service", "CarPlay mDNS publish and browse"),
];
