# Features

Statuses: Planned · Researching · In development · Implemented · Testing · Verified · Blocked ·
Deprecated. **Verified** only after the acceptance test ran on real hardware and is recorded in
MASTER_TESTING. "Upstream (LIVI)" = LIVI says it does this; open-car has not tested it yet.

## open-car's own features

| ID | Feature | Phase | Modules | Status | Tests | Done when |
|---|---|---|---|---|---|---|
| F-001 | Device profile format with honesty rules | 1 | `profile` | Implemented | unit (8) | — |
| F-002 | Compatibility database: load, validate, search, show | 1 | `compat`, `opencar compat` | Implemented | unit (3), CLI (2) | — |
| F-003 | Hardware probe: CPU/board/RAM/OS, USB host/gadget, phones on USB, Bluetooth, Wi-Fi + AP mode, displays + modes, touch, rotary, I2C, audio cards, GStreamer decoders, LIVI's tools | 1 | `probe` | Implemented (tested on fake trees and one cloud VM) | unit (6) | Readings confirmed on a Raspberry Pi and a PC with real radios and a screen |
| F-004 | Readiness check per feature (display, decoding, touch, wired/wireless AA, wired/wireless CarPlay) | 1 | `readiness`, `opencar check` | Implemented | unit (6), CLI (1) | Matches the outcome of real phone tests on 2 devices |
| F-005 | open-car settings (profile, LIVI Link, MFi bus) | 1 | `config`, `opencar setup` | Implemented | unit (3), CLI (2) | — |
| F-006 | Find LIVI's install (AppImage, config, autostart) | 1 | `livi::find_install`, `opencar livi status` | Implemented | unit (1) | Confirmed against a real LIVI install |
| F-007 | Write a profile's LIVI settings into `config.json` safely | 1 | `livi`, `opencar livi configure` | Implemented | unit (4), CLI (1) | LIVI reads the written settings on a real install |
| F-008 | Diagnostics file without personal data | 1 | `diag`, `opencar diag` | Implemented | unit (1), CLI (1) | — |
| F-009 | Function registry generated from code | 0 | `xtask` | Implemented | CI | — |
| F-010 | CI: format, lint, tests, registry, database, licences/advisories | 0 | `.github/workflows` | Implemented | runs on GitHub | First green run on GitHub |
| F-011 | Prebuilt `opencar` packages (.deb, arm64 + x86_64) and signed releases | 1 | — | Planned | — | — |
| F-012 | One-step installer: LIVI (pinned version) + opencar + profile | 1 | — | Planned | — | — |
| F-013 | Setup/diagnostics screen | 1–2 | — | Planned (ADR-0009, owner input) | — | — |
| F-014 | Hardware plugins per head unit (profile + LIVI settings + install steps) | 6 | — | Planned | — | — |
| F-015 | Public searchable compatibility site | 6 | — | Planned | — | — |
| F-016 | Android app | 5 | — | Researching (ADR-0010) | — | — |
| F-017 | Steering-wheel controls via an isolated adapter (e.g. resistive keypad interface) | 2+ | — | Planned (needs ADR + safety review) | — | — |

## Projection features (from LIVI)

| ID | Feature | Status for open-car |
|---|---|---|
| P-001 | Wired Android Auto | Upstream (LIVI), not yet tested by open-car |
| P-002 | Wireless Android Auto | Upstream (LIVI), not yet tested |
| P-003 | Wired CarPlay (with MFi chip or LIVI Link) | Upstream (LIVI), not yet tested; **Blocked** for us until we have an MFi chip or LIVI Link |
| P-004 | Wireless CarPlay | Same as P-003 |
| P-005 | Video: hardware decoding, any resolution, safe area | Upstream (LIVI), not yet tested |
| P-006 | Audio: media, navigation, calls, assistant; mic | Upstream (LIVI), not yet tested |
| P-007 | Touch, knob/D-pad, hard keys | Upstream (LIVI), not yet tested |
| P-008 | Auto-reconnect, several phones, live switching | Upstream (LIVI), not yet tested |
| P-009 | Day/night | Upstream (LIVI), not yet tested |
