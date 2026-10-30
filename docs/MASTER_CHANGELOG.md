# Changelog

Versions follow Semantic Versioning; releases are tagged `vX.Y.Z`.

## Unreleased: 0.1.0 (2026-10-30), Phase 0 + start of Phase 1

### Added
- Repository set-up: GPL-3.0-or-later licence, NOTICE with LIVI attribution, README, CLAUDE.md,
  CONTRIBUTING, CODE_OF_CONDUCT, SECURITY, issue and pull-request templates, CI.
- Research of existing projects (`docs/research/existing-projects.md`); LIVI chosen as the
  projection engine (ADR-0002); ADR-0001 to ADR-0011.
- `opencar-core`: device profiles with honesty rules, compatibility database, Linux hardware
  probe, readiness checks, LIVI install detection and safe `config.json` changes, open-car
  settings, diagnostics without personal data, atomic file writing.
- `opencar` CLI: `check`, `compat list|search|show|validate`, `setup`, `livi status|configure`, `diag`.
- Compatibility database with 4 untested profiles (generic x86_64 Linux, Raspberry Pi 5, two
  Scion FR-S factory units).
- `cargo xtask registry`: the generated function registry.
