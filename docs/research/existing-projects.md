# Existing open-source projection projects (research, 2026-10-30)

How this was checked: each repository was cloned and its README, licence, code layout and last
commit read on 2026-10-30. Feature claims are **the projects' own**, not tested by open-car.

## Summary

| Project | Language | Licence | Last commit | Protocols | Platforms | Verdict |
|---|---|---|---|---|---|---|
| **LIVI** (f-io/LIVI) | Rust (~134,000 lines), Slint-style UI, GStreamer | GPL-3.0-or-later + 7(b) attribution | 2026-10-29 (release v10.1.0) | CarPlay + Android Auto, wired + wireless | Linux x86_64/arm64 (Raspberry Pi incl.), macOS | **Foundation (ADR-0002)** |
| OpenAutoLink (mossyhub/openautolink) | Kotlin + C++ (aasdk via JNI) | GPL-3.0 | 2026-10-29 | Android Auto (wired + wireless), CarPlay (USB; wireless not working for its maintainer) | Android Automotive OS head units (GM) | **Candidate base for the Android app (ADR-0010)** |
| Headunit Revived / Open Headunit (andreknieriem/open-headunit) | Kotlin/Java | AGPL-3.0 | 2026-10-30 | Android Auto only | Android tablets/head units (Android 4.1+) | Candidate for non-AAOS Android head units (ADR-0010) |
| Crankshaft (opencardev/crankshaft) | Shell image builder + OpenAuto | GPL-3.0 | 2026-09-20 | Android Auto only | Raspberry Pi image | Reference for Pi image building and boot only |
| OpenAuto (opencardev/openauto fork) | C++ / Qt on aasdk | GPL-3.0 | 2026-02-08 | Android Auto only | Raspberry Pi, Linux | Superseded by LIVI for us |
| OpenAuto (f1xpl/openauto, original) | C++ / Qt on aasdk | GPL-3.0 | 2024-12-12 | Android Auto only | Raspberry Pi 3 | Unmaintained |
| aasdk (openDsh/aasdk) | C++ | GPL-3.0 | 2021-12-26 | Android Auto protocol library | Linux | Unmaintained in that fork |
| headunit (mikereidis, original) | Java | AGPL-3.0 | 2015-11-28 | Android Auto | Android | Abandoned; revived above |

## LIVI in detail

- **What it is:** a complete head unit: projection, its own launcher and settings, media player,
  DAB+ (RTL-SDR), GPS (NMEA/u-blox), dashboards/instrument cluster, multi-display, reverse camera.
- **CarPlay:** native accessory side (iAP2 crates `iap2-*`), wired over USB without OTG, wireless
  over LIVI's own Wi-Fi access point with Bluetooth handover; H.264/H.265, Siri, calls, mic,
  now-playing, turn-by-turn, multi-session. **Authentication:** an MFi coprocessor on I2C
  (`carPlayMfiI2cBus`, default 2; `carPlayMfiPowerGpio`, default 21) or a LIVI Link (a box with its
  own radios and authentication). It "neither ships nor bypasses" the chip. Without one,
  CarPlay is unavailable; Android Auto is unaffected.
- **Android Auto:** wired and wireless (`livi-aa`, `livi-aa-stack`, `livi-aa-proto`); wired
  limited to one device.
- **Media:** zero-copy GStreamer pipeline; decoders it looks for: `v4l2(sl)h264/h265dec`,
  `vah264/h265dec`, `nvh264/h265dec`, software `avdec_h264/h265`.
- **Install:** one script (apt/dnf/pacman) installs the AppImage to `~/LIVI/LIVI.AppImage`, a
  desktop autostart entry or, headless, `livi-kiosk.service` with tty1 autologin. Raspberry Pi
  boards need Debian 13 (Trixie). Settings in `~/.config/LIVI/config.json`.
- **Interfaces open-car can use without changing LIVI:** `config.json` keys documented in its
  README; telemetry over `livi-core telemetry …` or HTTP `POST :4000/telemetry` (gear, lights,
  climate, GPS) for a future read-only vehicle-data adapter.
- **Contributing upstream:** issue first; a Contributor Licence Agreement granting LIVI's author an
  **exclusive** licence to contributions; AI tools may not appear as commit author or
  `Co-authored-by` (an `Assisted-by:` trailer is allowed). See ADR-0011.
- **Risks:** one main maintainer (bus factor); fast-moving (v10.x); the 7(b) attribution term
  binds every redistribution.

**Recommendation (accepted by the owner 2026-10-30):** build on LIVI unmodified, do not fork, do
not write a second projection stack.

## Android

LIVI has no Android app. Two GPL-family Android receivers exist:
- **OpenAutoLink:** targets Android Automotive OS (Google's car OS), installed through a Play
  Console internal test on locked GM cars; CarPlay needs the user's own authorised
  authentication material, which its builder does not supply. Closest to "an app on the car's
  existing Android screen".
- **Headunit Revived:** Android Auto on ordinary Android tablets and the many Chinese Android
  head units (MTK/Rockchip settings in its wiki). It warns that Android Auto 17.4+ can block
  legacy wireless start-up; it offers several workarounds (dongle, Wi-Fi Direct, developer
  "headunit server", root).

Licence note: AGPL-3.0 code may be combined with GPL-3.0 code (section 13 of each), but the
combined work's network-interaction terms then apply to the AGPL part.

## Feasibility in one paragraph

On Linux (any x86_64/arm64 computer with USB host, Bluetooth, an AP-capable Wi-Fi card and a
screen), Android Auto is feasible today through LIVI. CarPlay is feasible only with an MFi
coprocessor or a LIVI Link. On Android head units, Android Auto is feasible through the projects
above with version-dependent caveats; CarPlay depends on authorised authentication. On closed
factory units (like the Scion FR-S's), nothing can be installed until shown otherwise: they need
an external adapter computer, and their screens may not accept outside video at all.

## Sources

- https://github.com/f-io/LIVI (README, NOTICE, CLA.md, CONTRIBUTING.md, `native/crates`, `scripts/install`)
- https://github.com/mossyhub/openautolink
- https://github.com/andreknieriem/open-headunit
- https://github.com/opencardev/crankshaft, https://github.com/opencardev/openauto
- https://github.com/f1xpl/openauto, https://github.com/openDsh/aasdk, https://github.com/mikereidis/headunit
