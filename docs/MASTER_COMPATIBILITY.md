# Compatibility

The database is `compat/profiles/*.toml`, one file per device, built into `opencar`.
`opencar compat list` shows it; `opencar compat validate` checks it (CI does too).

## Current entries

| Profile | Status | Install | Android Auto | CarPlay |
|---|---|---|---|---|
| `generic-linux-x86-64` | Untested | native Linux | unknown | unknown |
| `raspberry-pi-5` | Untested | external adapter | unknown | unknown (needs MFi chip or LIVI Link) |
| `scion-frs-pioneer-2013-2014` | Untested | unknown | unknown | unknown |
| `scion-frs-pioneer-display-audio-2015-2016` | Untested | unknown | unknown | unknown |

No device is verified yet.

## Statuses

`verified` (real test, evidence recorded) · `experimental` (runs, with gaps) · `untested` ·
`unsupported` · `requires-external-hardware` (the unit cannot run it; an adapter computer can).

## Profile fields

Required: `id` (= file name, lowercase, digits, dashes), `manufacturer`, `model`, `installation`
(`native-linux`, `native-android`, `external-adapter`, `none`, `unknown`), `android_auto` and
`carplay` (`yes`, `experimental`, `via-adapter`, `no`, `unknown`), `status`.

Optional, left out when unknown (never guessed): `vehicles`, `hardware_revision`,
`operating_system`, `firmware_version`, `cpu_arch`, `ram_mb`, `[screen]` (`width`, `height`,
`diagonal_in`), `usb`, `bluetooth`, `wifi`, `video_decode`, `audio`, `touchscreen`,
`physical_controls`, `carplay_auth`, `known_issues`, `notes`, `[livi]` (`mfi_i2c_bus`,
`mfi_power_gpio`, `gps_enabled`, `gps_device`, `gps_baud_rate`), and `[[evidence]]`
(`date`, `tester`, `software`, `result`, `link`).

## Rules (enforced)

See ADR-0005: `verified` and any `yes` need evidence; CarPlay claims need `carplay_auth`;
`requires-external-hardware` needs `installation = "external-adapter"`; `untested` and
`unsupported` cannot claim working features; ids are unique and match the file name.

## Adding a device

[COMPATIBILITY_REPORTS.md](COMPATIBILITY_REPORTS.md). Community reports are reviewed by a
maintainer before a profile's status changes.

## Platform facts that limit compatibility

- Closed factory head units usually cannot run third-party software; they need an external
  adapter, and their screens may not accept outside video.
- Wireless projection needs a Wi-Fi interface that supports access-point mode and Bluetooth (or a
  LIVI Link).
- CarPlay needs Apple MFi authentication hardware (ADR-0008).
- Raspberry Pi boards need Raspberry Pi OS based on Debian 13 for LIVI.
