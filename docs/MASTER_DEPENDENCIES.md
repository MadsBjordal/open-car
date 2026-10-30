# Dependencies

Every dependency must be GPL-3.0-compatible (`deny.toml`, checked in CI with cargo-deny).
Versions are pinned by `Cargo.lock`.

## Runtime

| Dependency | Version | Licence | Why |
|---|---|---|---|
| **LIVI** (separate program, installed by its own installer) | tested against v10.1.0; code read at `main` e7b20b8 (2026-10-29) | GPL-3.0-or-later + 7(b) attribution | The projection engine (ADR-0002) |
| serde | 1.0.229 | MIT OR Apache-2.0 | Reading/writing profiles, settings, JSON |
| serde_json | 1.0.154 | MIT OR Apache-2.0 | LIVI's `config.json`, diagnostics |
| toml | 1.1.8 | MIT OR Apache-2.0 | Profiles and open-car settings |
| thiserror | 2.0.21 | MIT OR Apache-2.0 | Error types |
| clap | 4.6.7 | MIT OR Apache-2.0 | Command-line parsing |

## Development only

| Dependency | Version | Licence | Why |
|---|---|---|---|
| tempfile | 3.27.0 | MIT OR Apache-2.0 | Temporary folders in tests |
| Rust toolchain | stable, edition 2024, minimum 1.85 | MIT OR Apache-2.0 | Built and tested with rustc 1.97.0 |

The whole tree (50 crates from crates.io, 2026-10-30): MIT OR Apache-2.0 (most), MIT, Unlicense OR
MIT, Apache-2.0 WITH LLVM-exception, Unicode-3.0, LGPL-2.1-or-later (as an option). All compatible.

## System programs read by the probe (optional)

`iw` (Wi-Fi access-point mode), `gst-inspect-1.0` (decoders). Without them the check says
"unknown" rather than guessing.
