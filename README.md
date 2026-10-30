# open-car

**Android Auto and Apple CarPlay for cars that never had them**, using the car's existing head unit
where it can run software, or a small adapter computer (such as a Raspberry Pi) where it cannot.

Based on [LIVI](https://github.com/f-io/LIVI) by Lasse Heitgres: LIVI is the projection engine.
open-car adds what LIVI does not have: a hardware compatibility database, a hardware check, the
LIVI setup for each device, and (planned) installers, a hardware-profile system and an Android app.

> **Status: Phase 1, early.** The `opencar` tool works on any Linux computer. Nothing has been
> tested in a car yet, and no device is marked as verified. See
> [docs/MASTER_ROADMAP.md](docs/MASTER_ROADMAP.md).

## What works today

| Command | What it does |
|---|---|
| `opencar check` | Looks at this computer (USB, Bluetooth, Wi-Fi access-point mode, screens, touch, video decoders, I2C, the tools LIVI needs) and says, per feature, what is ready or missing |
| `opencar compat search fr-s` | Searches the compatibility database by maker, model or car |
| `opencar compat list` / `show <id>` / `validate` | Lists, shows and checks the database |
| `opencar setup --profile raspberry-pi-5 --mfi-bus 1` | Saves which device this is and how CarPlay authenticates here |
| `opencar livi status` | Is LIVI installed, does it start by itself |
| `opencar livi configure [--dry-run]` | Writes the device's settings into LIVI's `config.json`, keeping everything else |
| `opencar diag` | Writes a diagnostics file for a report (stays on your computer, nothing is uploaded, no serial numbers or addresses) |

## CarPlay needs Apple authentication

Every CarPlay head unit must prove itself to the iPhone with an Apple **MFi authentication chip**.
open-car and LIVI never bypass this. You need either an MFi coprocessor wired to the computer's
I2C bus, or a [LIVI Link](https://github.com/f-io/LIVI/blob/main/LIVI-LINK.md). Android Auto needs
neither.

## Your car is safe from it

open-car never connects to the engine computer (ECU), the OBD port or the CAN bus, and never
changes a tune. See [docs/MASTER_SECURITY.md](docs/MASTER_SECURITY.md).

## Build and try it (Linux)

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh   # Rust, once
git clone https://github.com/MadsBjordal/open-car && cd open-car
cargo build --release
./target/release/opencar check
```

Then install LIVI itself with LIVI's installer: [docs/INSTALL_LINUX.md](docs/INSTALL_LINUX.md).

## Documents

[Project](docs/MASTER_PROJECT.md) · [Architecture](docs/MASTER_ARCHITECTURE.md) ·
[Decisions](docs/MASTER_DECISIONS.md) · [Features](docs/MASTER_FEATURES.md) ·
[Function registry](docs/MASTER_FUNCTIONS.md) · [Compatibility](docs/MASTER_COMPATIBILITY.md) ·
[Dependencies](docs/MASTER_DEPENDENCIES.md) · [Security](docs/MASTER_SECURITY.md) ·
[Testing](docs/MASTER_TESTING.md) · [Roadmap](docs/MASTER_ROADMAP.md) · [Issues](docs/MASTER_ISSUES.md) ·
[Changelog](docs/MASTER_CHANGELOG.md) · [Research](docs/research/existing-projects.md)

## Licence

GPL-3.0-or-later. See [LICENSE](LICENSE) and [NOTICE](NOTICE). Contributions: [CONTRIBUTING.md](CONTRIBUTING.md).
