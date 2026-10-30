# open-car: rules for every development session (people and Claude)

open-car adds Android Auto and Apple CarPlay to existing car head units. **The projection engine is
[LIVI](https://github.com/f-io/LIVI)** (ADR-0002); open-car is the layer around it: hardware
profiles and the compatibility database, hardware checks, LIVI configuration, installation,
diagnostics, and later the Android app. Licence GPL-3.0-or-later (ADR-0007).

## Start of every session

1. Read this file, then `docs/MASTER_ROADMAP.md` (where we are), `docs/MASTER_ISSUES.md` (open
   problems) and the decisions in `docs/MASTER_DECISIONS.md` that touch your task.
2. `git log --oneline -20` and `git status`: continue from the repository's real state, never from
   what a conversation remembers.
3. Before writing code, search `docs/MASTER_FUNCTIONS.md` (generated from the code, so it is
   true) and the code itself. Order: **REUSE → EXTEND → REFACTOR → CREATE NEW**. Something that
   already does most of the job is extended, not copied. A new module needs a reason, written in
   the commit message or an ADR.
4. Before adding anything LIVI might already do, check LIVI (README, `native/crates/`). We never
   re-implement what LIVI does; we configure it, or improve it upstream (ADR-0011).

## Rules

1. **Decisions:** every significant choice is an ADR in `docs/MASTER_DECISIONS.md` (next free
   number, status Proposed/Accepted/Superseded/Deprecated/Rejected). Never silently contradict
   an accepted ADR: write a new one that supersedes it.
2. **Registry:** every `pub` item gets a `///` comment whose first sentence says what it is for.
   After adding or renaming, run `cargo xtask registry` and commit `docs/MASTER_FUNCTIONS.md`
   (CI fails when it is stale or a comment is missing).
3. **Docs follow the code in the same commit:** MASTER_FEATURES (status), MASTER_ARCHITECTURE,
   MASTER_TESTING (what ran), MASTER_CHANGELOG, MASTER_ISSUES, MASTER_ROADMAP,
   MASTER_DEPENDENCIES (new crate = name, version, licence, why), MASTER_SECURITY as affected.
   Docs describe what the software does now, not what it will do.
4. **Honesty:** never call unfinished work done, never claim a test passed that did not run,
   never present simulated or untested behaviour as working on hardware. A compatibility
   profile is "verified" only with recorded evidence (enforced by `opencar compat validate`).
   "Ready" from `opencar check` means requirements are present, not that it works.
5. **CarPlay:** only through Apple MFi authentication (a coprocessor or a LIVI Link). Never
   bypass, emulate or extract authentication; never commit MFi keys, certificates or firmware.
6. **Vehicle safety:** open-car never connects to the ECU, the OBD port or a CAN bus, never
   changes a tune (the reference car runs an EcuTek tune), and never controls the vehicle. Any
   future read-only vehicle data needs its own ADR, a separate isolated adapter and the
   owner's OK.
7. **Secrets and privacy:** nothing secret in git (`.env`, keys, MFi material). The probe and
   diagnostics never collect MAC addresses, serial numbers, VINs, host names, contacts, calls
   or locations. Nothing is uploaded anywhere.
8. **Licences:** only GPL-3.0-compatible dependencies; record each in MASTER_DEPENDENCIES.
   Anything containing LIVI code keeps "Based on LIVI by Lasse Heitgres
   (https://github.com/f-io/LIVI)" (LIVI's NOTICE, GPL-3.0 section 7(b)).
9. **Checks before every commit** (CI runs the same):
   `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`,
   `cargo test --workspace`, `cargo xtask registry --check`, `cargo run -q -p opencar -- compat validate`.
10. **Git:** feature branches, pull requests into `main`, messages in the imperative
    ("Add …", "Fix …"). Never rewrite published history. Versions follow SemVer, tagged `vX.Y.Z`.
11. **Ask the owner first** for: architecture changes, licence changes, paid or proprietary
    dependencies, cloud services, removing features, anything that controls the vehicle,
    security trade-offs, required hardware purchases, releases and repository visibility.
    Bring a recommendation. Routine choices: decide and record.
12. **End of a session:** update the docs above, commit, and report: what was built, what was
    reused/extended/created and why, tests run and their results, tests not run, open issues,
    next step.

## The owner

Not a professional developer; uses Windows. Explain choices in plain language. At the end of
each milestone say exactly how to run it and what to test by hand.

## Layout

- `crates/opencar-core`: the library. `profile` (device profile + honesty rules), `compat`
  (the database), `probe` (hardware abstraction layer: reads `/sys`, `/proc`, `/dev` under a root;
  `CommandRunner` for outside programs), `readiness` (what LIVI needs vs what is here),
  `livi` (LIVI's paths and `config.json`), `config` (open-car's settings), `diag`, `fsutil`
  (`write_atomic`).
- `crates/opencar`: the `opencar` command-line tool; `build.rs` builds `compat/profiles` in.
- `crates/xtask`: `cargo xtask registry`.
- `compat/profiles/*.toml`: the compatibility database (schema: `docs/MASTER_COMPATIBILITY.md`).
- `docs/`: the master documents and `docs/research/`.
