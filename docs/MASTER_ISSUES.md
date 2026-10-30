# Known issues and limitations

| ID | Issue | Impact | Status |
|---|---|---|---|
| I-001 | Nothing tested on real hardware or in a car | All projection features are LIVI's claims, unconfirmed by us | Open: Phase 2 |
| I-002 | CarPlay needs an MFi coprocessor or a LIVI Link; we have neither | CarPlay cannot be tested | Open: hardware purchase needs the owner's OK |
| I-003 | `opencar livi configure` while LIVI runs: LIVI may write its own settings back over ours | Settings lost | Open: documented ("stop LIVI first"); detect a running LIVI later |
| I-004 | Wi-Fi access-point support is read with `iw`; without `iw` it shows "unknown" | Less precise check | Accepted (LIVI's installer adds `iw`) |
| I-005 | LIVI has one main maintainer and moves fast (v10.x) | Interfaces we rely on (config keys, paths) may change | Open: pin the tested release; watch upstream |
| I-006 | The MFi chip on I2C cannot be confirmed by open-car (it does not talk to the bus) | `check` says "unknown" for CarPlay with a configured bus | Accepted by design (ADR-0003) |
| I-007 | Touch/rotary detection reads `/proc/bus/input/devices` heuristically (direct-input property; REL_DIAL without REL_X) | Unusual devices may be missed | Open: confirm on real hardware |
| I-008 | The Scion FR-S head unit is not identified | Its profiles say "unknown" everywhere | Open: owner to photograph it |
| I-009 | No prebuilt packages: users must install Rust and build | Harder install | Open: F-011 |
