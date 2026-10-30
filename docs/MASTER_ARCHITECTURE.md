# Architecture

```
            Phone (Android Auto / CarPlay)
               │  USB, or Bluetooth + Wi-Fi
┌──────────────▼──────────────────────────────────────────┐
│ LIVI (upstream, unmodified; ADR-0002)                    │
│  projection (AA, CarPlay/iAP2), GStreamer video, audio,  │
│  input, Wi-Fi AP + Bluetooth, launcher UI, settings      │
│  ~/.config/LIVI/config.json   ~/LIVI/LIVI.AppImage       │
└──────────────▲──────────────────────────────────────────┘
               │ documented config keys only (ADR-0004)
┌──────────────┴──────────────────────────────────────────┐
│ opencar (CLI)  check · compat · setup · livi · diag      │
├──────────────────────────────────────────────────────────┤
│ opencar-core                                             │
│  profile ── compat (database)        config (open-car's) │
│  probe (HAL read side) ── readiness   livi   diag        │
│  fsutil::write_atomic (every save)                       │
└──────────────▲───────────────────────────▲──────────────┘
               │ reads only                 │ built in at compile time
   /sys /proc /dev, iw, gst-inspect   compat/profiles/*.toml
```

Never connected: ECU, OBD-II, CAN bus (MASTER_SECURITY).

## Data flows

- **`opencar check`:** `probe::probe(root, &SystemCommands)` → `HardwareReport` →
  `readiness::assess(report, Options from config)` → `Vec<Check>` → text or redacted JSON.
- **`opencar livi configure`:** `Config::load` → profile from the built-in `CompatDb` →
  `LiviSettings` (+ MFi bus from config) → `livi::plan_settings` / `apply_settings` →
  `write_atomic` with `.bak`.
- **Database:** `build.rs` embeds `compat/profiles/*.toml`; `CompatDb::from_sources` parses them;
  `--dir` reads a folder instead (for contributors' new profiles).

## Module dependencies

`profile` ← `compat` · `probe` ← `readiness` · `profile` ← `livi` · `probe`, `readiness`, `livi` ←
`diag` · `fsutil` ← `config`, `livi`, CLI. The CLI depends only on `opencar-core`.

## Boundaries

- Projection, media, input and radios: LIVI. open-car adds no second implementation.
- Hardware access: `probe` reads; it never writes devices. Future write-side adapters (e.g.
  steering-wheel keypad interface) get their own module and ADR.
- Platform split: everything Linux-specific is in `probe/linux.rs` and `livi.rs` paths. The
  Android app (Phase 5) reuses the profile format and rules, not the Linux probe.

## Storage

| What | Where | Written by |
|---|---|---|
| open-car settings | `$XDG_CONFIG_HOME/open-car/config.toml` (else `~/.config/…`), `.bak` kept | `Config::save` |
| LIVI settings | `$XDG_CONFIG_HOME/LIVI/config.json`, `.bak` kept | `livi::apply_settings` |
| Diagnostics | where the user says (`-o`) | `opencar diag` |
