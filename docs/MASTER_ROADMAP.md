# Roadmap

**Now:** Phase 1 (Linux foundation), first part done on 2026-10-30.

| Phase | Content | Status |
|---|---|---|
| 0 | Repository, master documents, research, licence, architecture, reference hardware | **Done** (2026-10-30). Reference hardware chosen on paper: Raspberry Pi 5; to be bought by the owner |
| 1 | Linux foundation: core library, hardware check, compatibility database, LIVI configuration, diagnostics, CI | **In progress**: code done and tested off-hardware. Left: first green CI run, prebuilt packages (F-011), installer (F-012), decision on a setup screen (ADR-0009) |
| 2 | Wired Android Auto on test hardware (through LIVI), measured | Not started: needs a Raspberry Pi 5 (or a Linux PC), a screen and an Android phone |
| 3 | Wireless Android Auto, saved phones, reliability | Not started |
| 4 | CarPlay | Blocked until an MFi coprocessor or LIVI Link is available |
| 5 | Android app (ADR-0010) | Research done; not started |
| 6 | Compatibility ecosystem: public database site, reports, hardware plugins | Format and validation done; the rest not started |
| 7 | Stable public release | Not started |

## Next tasks, in order

1. Merge the first pull request and see CI pass on GitHub.
2. Owner: get a Raspberry Pi 5 (8 GB), a 7" HDMI/DSI touchscreen, a 27 W USB-C supply for the desk,
   and photograph the FR-S head unit (research/reference-vehicle.md).
3. Run the Phase 2 hardware test plan (MASTER_TESTING) and record evidence in `raspberry-pi-5.toml`.
4. Prebuilt `opencar` .deb for arm64 and x86_64 from CI, attached to tagged releases (F-011).
5. One-step installer that installs a pinned LIVI release and opencar, then runs `opencar check` (F-012).
6. Review LIVI's telemetry endpoint and access point security (S-001).
