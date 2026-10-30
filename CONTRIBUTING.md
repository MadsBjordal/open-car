# Contributing to open-car

Thank you for helping. Read [CLAUDE.md](CLAUDE.md): its rules apply to every contributor, human or AI.

## The most useful contribution: your head unit

Run `opencar check` and `opencar diag` on your device and open a **Compatibility report** issue.
See [docs/COMPATIBILITY_REPORTS.md](docs/COMPATIBILITY_REPORTS.md). A profile is only marked
*verified* after a maintainer has reviewed real test evidence.

## Code

1. Open an issue first for anything larger than a small fix.
2. Search `docs/MASTER_FUNCTIONS.md` and the code before adding anything: reuse or extend what exists.
3. Branch from `main`, keep one topic per pull request, fill in the template.
4. Run the checks (CI runs the same):
   ```bash
   cargo fmt --all --check
   cargo clippy --workspace --all-targets -- -D warnings
   cargo test --workspace
   cargo xtask registry --check
   cargo run -q -p opencar -- compat validate
   ```
5. Every `pub` item needs a `///` comment; run `cargo xtask registry` after adding one.
6. Update the master documents your change affects, in the same pull request.

## Rules that are never bent

- No code that bypasses, emulates or extracts Apple MFi authentication; no MFi keys,
  certificates, firmware images or manufacturer files in the repository.
- No connection to an ECU, OBD port or CAN bus, and nothing that controls a vehicle.
- No collection or upload of personal data.
- Dependencies must be GPL-3.0-compatible.

## Licence

By contributing you agree that your contribution is licensed under GPL-3.0-or-later.

## Improvements to LIVI

Changes that belong in the projection engine go to [LIVI](https://github.com/f-io/LIVI) itself,
under LIVI's own contribution rules (an issue first, its CLA, its rules on AI-assisted work).
