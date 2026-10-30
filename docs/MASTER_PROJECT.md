# Master project document

## Purpose

open-car brings **Android Auto and Apple CarPlay** to vehicles whose factory or aftermarket head
units never had them: on the head unit itself when it can run Linux or Android software, otherwise
on a small external adapter computer. It keeps the original hardware wherever that is technically
possible, and says plainly where it is not.

## Scope

- **In:** a universal infotainment layer around an existing projection engine: hardware
  compatibility (profiles, database, detection), installation and updates, device configuration,
  diagnostics, an Android app for Android head units, hardware-specific plugins.
- **Not in:** replacing proprietary head-unit firmware (Phase 0 decision; the architecture leaves
  room for it later), re-implementing Google Maps/Spotify/etc. (the phone projects them),
  anything that controls the vehicle.
- **Projection engine:** [LIVI](https://github.com/f-io/LIVI) (ADR-0002), used unmodified.

## Principles

Open source (GPL-3.0-or-later, ADR-0007) · reuse before writing (CLAUDE.md) · honest status
(ADR-0005) · legitimate CarPlay authentication only (ADR-0008) · no vehicle-system access
(MASTER_SECURITY) · privacy by default · stability over features · documentation as the single
source of truth.

## Requirements (from the owner's brief, 2026-10-30)

Supported where hardware permits: CarPlay; Android Auto; wired USB; wireless (Bluetooth + Wi-Fi);
navigation, media, calls and voice assistants through the phone; touch, buttons, rotary and
steering-wheel controls; automatic reconnection; several phone profiles; any screen size; day/night;
audio out and microphone in; settings; hardware compatibility detection; simple install and update;
community hardware support.

Where these stand: [MASTER_FEATURES.md](MASTER_FEATURES.md). Most projection features come from
LIVI and are marked "Upstream (LIVI), not yet tested by open-car" until we test them.

## Reference vehicle

2014 Scion FR-S ([research/reference-vehicle.md](research/reference-vehicle.md)): a case study only.
Its EcuTek engine tune is never touched.

## The owner

Mads Bjordal: decides architecture, licence, costs, security trade-offs, releases. Not a
professional developer; works on Windows.

## Relation to `86-infotainment`

A separate, earlier repository with an FR-S-specific Python prototype. open-car is independent of
it (ADR-0001); nothing is shared.
