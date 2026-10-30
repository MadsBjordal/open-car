# Security

## Hard rules

1. **Vehicle isolation.** open-car never connects to the ECU, the OBD-II port or any CAN bus;
   never reads, flashes or changes an engine tune (the reference car runs EcuTek), the
   immobiliser, ABS, stability control or airbags; never controls the vehicle. The car must drive
   exactly the same with open-car unplugged, crashed or dead. Future read-only vehicle data
   needs its own ADR, a separate isolated adapter and the owner's approval.
2. **Apple authentication** is never bypassed, emulated or extracted (ADR-0008). No MFi keys,
   certificates or firmware in the repository (`.gitignore` blocks `mfi/`, `*.pem`, `*.key`, `*.der`).
3. **Privacy.** The probe never reads MAC addresses, serial numbers, host names, VINs, contacts,
   calls, messages or locations. Diagnostics stay on the device; the home folder is shown as `~`.
   Nothing is uploaded and there is no telemetry. Any optional upload in future needs explicit
   consent each time.
4. **No secrets in git.** `.env`, keys and diagnostics files are ignored.
5. **No network services.** open-car opens no ports. (LIVI runs its own access point, mDNS and a
   telemetry endpoint on port 4000; see risks.)

## Current design measures

- Rust with `unsafe_code = "forbid"` across the workspace.
- Probing only reads files and runs two read-only programs (`iw phy … info`,
  `gst-inspect-1.0 --exists …`) with fixed arguments; no shell.
- Every saved file goes through `write_atomic` (no half-written settings after a power cut).
- `livi configure` refuses a LIVI `config.json` that is not a JSON object and leaves it untouched.
- CI: `cargo deny` (licences, known vulnerabilities from the RustSec database, crate sources),
  clippy with warnings as errors.

## Known risks

| ID | Risk | Status |
|---|---|---|
| S-001 | LIVI's telemetry HTTP endpoint (port 4000) and its Wi-Fi access point are reachable by devices on that network | To review in LIVI's code before Phase 2; document how to firewall it |
| S-002 | LIVI's installer runs with `sudo` and downloads the latest build | Owner/user reads it first (INSTALL_LINUX); a pinned, checksum-verified install is planned (F-012) |
| S-003 | Release signing not set up | Planned with F-011 |
| S-004 | A car's 12 V supply (cranking dips, load dump) can corrupt storage | Atomic saves done; the power supply choice is a hardware task |

## Reviews

| Date | Scope | Result |
|---|---|---|
| 2026-10-30 | Phase 0/1 code, by the author (no independent review) | No network code; no secrets; probe output checked for leaks by test `report_holds_no_serials_or_addresses` |
