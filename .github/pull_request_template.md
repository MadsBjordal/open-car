## What and why

## Reuse check
- [ ] I searched `docs/MASTER_FUNCTIONS.md` and the code; this reuses / extends: …
- [ ] New modules or functions, and why existing ones could not be extended: …

## Checks
- [ ] `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`
- [ ] `cargo xtask registry` run and `docs/MASTER_FUNCTIONS.md` committed
- [ ] Master documents updated (features, testing, changelog, issues, decisions as affected)
- [ ] Tested on real hardware: (device, phone, what) / not tested on hardware

## Safety
- [ ] No ECU/OBD/CAN access, no Apple authentication bypass, no secrets or personal data
