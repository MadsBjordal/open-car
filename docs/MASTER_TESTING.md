# Testing

## How to run

```bash
cargo test --workspace            # unit + end-to-end CLI tests
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all --check
cargo xtask registry --check
cargo run -q -p opencar -- compat validate
cargo deny check                  # needs `cargo install cargo-deny`
```

## Kinds of tests

| Kind | Where | Hardware? |
|---|---|---|
| Unit | each file's `mod tests` in `opencar-core` and `opencar` | No: fake `/sys` trees and a fake `CommandRunner` |
| End-to-end CLI | `crates/opencar/tests/cli.rs` (runs the real binary with a temporary home) | No |
| Regression | the whole suite in CI on every push and pull request | No |
| Hardware | manual, recorded below and as `[[evidence]]` in profiles | **Yes**: none done yet |
| Connectivity / performance | Phase 2, on real hardware; targets set after baseline measurements | Not started |
| Security | `cargo deny`, clippy, leak test | No |

## Results

| Date | What ran | Where | Result |
|---|---|---|---|
| 2026-10-30 | `cargo test --workspace`: 43 tests (35 core unit, 3 CLI unit, 5 end-to-end) | Cloud Linux VM, x86_64, rustc 1.97.0 | All passed |
| 2026-10-30 | `cargo clippy -D warnings`, `cargo fmt --check`, `cargo xtask registry --check`, `opencar compat validate` | same | Passed |
| 2026-10-30 | `cargo deny check` (licences, RustSec advisories, sources); build with Rust 1.85.1 (minimum version) | same | Passed |
| 2026-10-30 | `opencar check` on the VM itself (no USB, radios or screen) | same | Ran; reported the missing hardware correctly. Not a hardware test |

**Not tested yet:** anything on a Raspberry Pi, a head unit, with a phone, with LIVI installed, or
with real Wi-Fi/Bluetooth/screens. CI on GitHub has not run yet at the time of writing.

## Hardware test plan (Phase 2)

1. `opencar check` on a Raspberry Pi 5 with Raspberry Pi OS (Trixie): compare every line with
   the real hardware (`lsusb`, `iw list`, `bluetoothctl list`, `gst-inspect-1.0`).
2. Install LIVI v10.1.0; `opencar livi status` must find it; `opencar livi configure` must
   change only the chosen keys and LIVI must start with them.
3. Wired Android Auto with two phones (different Android versions): connect, video, touch,
   music, navigation voice, call with mic, unplug/replug, reboot with phone attached.
4. Wireless Android Auto: pairing, reconnect after power cycle, two saved phones.
5. Record times (start-up, connect, reconnect) and CPU/RAM as baselines; set targets from them.
