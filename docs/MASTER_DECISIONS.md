# Architecture decision records

Status: Proposed · Accepted · Superseded · Deprecated · Rejected. Read the relevant ones before
changing anything; never contradict an Accepted one silently: add a new record that supersedes it.

---

## ADR-0001 — A dedicated repository, independent of other projects

- **Date:** 2026-10-30 · **Status:** Accepted
- **Context:** The owner's other repositories include `86-infotainment`, an FR-S-specific Python
  prototype whose decision D5 ruled phone projection out. The brief asks for a dedicated,
  independent repository.
- **Decision:** All work lives in `MadsBjordal/open-car` (created by the owner). No code, docs or
  decisions are taken over from `86-infotainment`; its FR-S research is summarised fresh in
  `docs/research/reference-vehicle.md`.
- **Alternatives:** grow `86-infotainment` (rejected: FR-S-specific scope, Python stack, and a
  decision against projection).
- **Consequences:** two repositories about car infotainment exist; `86-infotainment` is not
  maintained by this project.
- **Affected:** everything. **Related:** ADR-0002.

## ADR-0002 — LIVI is the projection engine; open-car does not write its own

- **Date:** 2026-10-30 · **Status:** Accepted (owner's choice, 2026-10-30)
- **Context:** The brief proposes a Rust core with Android Auto and CarPlay receivers. LIVI
  (GPL-3.0-or-later, ~134,000 lines of Rust, last commit 2026-10-29, release v10.1.0) already
  implements wired and wireless CarPlay and Android Auto on Linux, with GStreamer hardware
  decoding, audio, mic, calls, touch/knob input, multi-session, reconnection, its own launcher
  and settings. See `research/existing-projects.md`.
- **Decision:** Use LIVI unmodified, installed by its own installer, as the projection engine
  and head-unit UI. open-car builds what LIVI lacks around it.
- **Alternatives:** fork LIVI (rejected: 134k lines to maintain, drift from upstream fixes); own
  Rust stack (rejected: years of duplicate work, against the reuse rule); OpenAuto/aasdk (C++,
  Android Auto only, older).
- **Justification:** maximum reuse, active upstream, matches the brief's preferred stack (Rust,
  GStreamer, BlueZ, Linux).
- **Consequences:** open-car's Linux features depend on LIVI's releases and interfaces; LIVI's
  licence terms apply to any bundle (ADR-0007); fixes to projection go upstream (ADR-0011);
  single-maintainer risk upstream (MASTER_ISSUES I-005).
- **Affected:** all crates. **Related:** ADR-0003, 0004, 0009, 0011.

## ADR-0003 — open-car core in Rust; hardware behind a probe interface

- **Date:** 2026-10-30 · **Status:** Accepted
- **Context:** open-car needs hardware detection and configuration that works on any Linux
  computer and can be tested without hardware.
- **Decision:** A Rust library `opencar-core` and a CLI `opencar`. The hardware abstraction
  layer's read side is `probe`: it reads `/sys`, `/proc` and `/dev` under a root folder (a fake
  tree in tests) and runs outside programs through the `CommandRunner` trait. It only reads; it
  never writes to devices or the I2C bus.
- **Alternatives:** Python (rejected: second language next to LIVI's Rust); shelling out to
  `lsusb`/`iw` for everything (rejected: more dependencies, harder to test).
- **Consequences:** probing is fully unit-tested against fake trees; real hardware still has to
  confirm the reading (MASTER_TESTING). Write-side HAL (display, audio routing, controls) stays
  in LIVI.

## ADR-0004 — open-car changes LIVI only through its documented settings

- **Date:** 2026-10-30 · **Status:** Accepted
- **Decision:** open-car writes only keys documented in LIVI's README (`carPlayMfiI2cBus`,
  `carPlayMfiPowerGpio`, `gpsEnabled`, `gpsDevice`, `gpsBaudRate` so far) into
  `~/.config/LIVI/config.json`, keeping every other key, atomically, with a `.bak` copy. It never
  patches LIVI's files. New keys are added only after reading LIVI's source for them.
- **Consequences:** LIVI should be stopped while configuring (MASTER_ISSUES I-003).

## ADR-0005 — Honest compatibility: no "verified" without evidence

- **Date:** 2026-10-30 · **Status:** Accepted
- **Decision:** Profiles in `compat/profiles/*.toml` are checked by `DeviceProfile::validate`
  (CI runs `opencar compat validate`): `verified` and any `yes` need `[[evidence]]`; CarPlay
  claims need `carplay_auth`; `requires-external-hardware` needs `installation =
  external-adapter`; `untested`/`unsupported` cannot claim working features. `opencar check`
  says "ready" only for present requirements, never "works".

## ADR-0006 — The function registry is generated from the code

- **Date:** 2026-10-30 · **Status:** Accepted
- **Decision:** `cargo xtask registry` writes `docs/MASTER_FUNCTIONS.md` from every source file:
  modules (with `//!` purpose and test count) and public items (id, kind, purpose = first
  sentence of `///`, signature). `--check` in CI fails when it is stale or a public item has no
  comment. Feature, ADR and test links live in MASTER_FEATURES and the comments themselves.
- **Alternatives:** a hand-kept registry (rejected: goes out of date).

## ADR-0007 — Licence GPL-3.0-or-later, with LIVI's attribution

- **Date:** 2026-10-30 · **Status:** Accepted (owner's choice)
- **Context:** every usable projection project is GPL-3.0 (Headunit Revived: AGPL-3.0).
- **Decision:** open-car is GPL-3.0-or-later. `NOTICE` and the README carry "Based on LIVI by
  Lasse Heitgres (https://github.com/f-io/LIVI)"; any future UI's About screen will too.
  Dependencies must be GPL-3.0-compatible (enforced by `cargo deny` with `deny.toml`).

## ADR-0008 — CarPlay only through Apple MFi authentication

- **Date:** 2026-10-30 · **Status:** Accepted
- **Decision:** CarPlay is supported only with an MFi coprocessor (I2C, configured in LIVI) or a
  LIVI Link. open-car never bypasses, emulates or extracts authentication and never stores MFi
  keys, certificates or firmware. Without authentication, CarPlay is shown as unavailable and
  Android Auto work continues independently.

## ADR-0009 — No separate open-car driving interface in Phase 1

- **Date:** 2026-10-30 · **Status:** Proposed
- **Context:** The brief asks for a launcher (Android Auto, CarPlay, devices, media, settings,
  diagnostics, about). LIVI already has a launcher, device list, media, settings and day/night.
- **Decision (proposed):** The head-unit UI is LIVI's. open-car's interface in Phase 1 is the
  `opencar` CLI for setup and diagnostics. A small setup/diagnostics screen (e.g. a LIVI
  extension or a separate Slint app) is decided later, when real installs show what is missing.
- **Owner input wanted:** whether a separate open-car screen is wanted at all.

## ADR-0010 — Android app built on an existing GPL receiver

- **Date:** 2026-10-30 · **Status:** Proposed
- **Decision (proposed):** For Phase 5, evaluate OpenAutoLink (GPL-3.0, Android Automotive OS,
  Android Auto + USB CarPlay) and Headunit Revived (AGPL-3.0, Android Auto on ordinary Android
  head units) as the base, sharing open-car's profile format and rules. Not decided until both
  are tried on a real Android head unit.

## ADR-0011 — Improvements to LIVI go upstream, only with the owner's OK

- **Date:** 2026-10-30 · **Status:** Proposed
- **Context:** LIVI requires a CLA granting its author an exclusive licence to contributions, and
  does not accept AI tools as commit author or co-author.
- **Decision (proposed):** Projection fixes are proposed upstream as issues first. A pull request
  to LIVI needs the owner's OK per change (the owner signs the CLA, commits carry an
  `Assisted-by:` trailer instead of a co-author line).
