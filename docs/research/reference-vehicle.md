# Reference vehicle: 2014 Scion FR-S

A case study, not a design limit (MASTER_PROJECT.md). Facts are **Unverified** unless marked.

| Item | Value | Status |
|---|---|---|
| Model | Scion FR-S, North American, MY2014 | Confirmed by the owner (VIN read in the earlier 86-infotainment project; the VIN is never stored) |
| Related cars | Toyota GT86/86, Subaru BRZ (same platform) | Well known |
| Engine | Supercharged/turbocharged, EcuTek tune | Confirmed by the owner. **open-car never touches it.** |
| Head unit | 2013-2014 Pioneer-made Scion unit (non-touch), or the optional 5.8" BeSpoke | Unverified: needs photos of the front, its version screen and the chassis label |
| Third-party software on it | No public method known (searched 2026-10-30) | Unverified |
| Video input | Unknown, probably none | Unverified |
| GPS | None in the factory units | Unverified |

## What this means

The most likely route is an **external adapter**: a Raspberry Pi (or similar) running LIVI and
open-car with its own screen in the dash opening, the factory unit removed whole and kept. That is
not decided until the real unit is identified. Driving the factory screen from a Pi would need its
panel and touch controller documented, which nobody has published.

## Next steps (owner)

1. Photograph the head unit's front, its version/system screen and the label on its case.
2. Note whether the car has a reverse camera and steering-wheel buttons.

Earlier research for this car lives in the separate repository `86-infotainment`.
