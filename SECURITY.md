# Security policy

## Reporting a vulnerability

Please do **not** open a public issue. Use GitHub's private
[security advisory](https://github.com/MadsBjordal/open-car/security/advisories/new) form. Expect
a first answer within 7 days. Problems in the projection engine itself belong to
[LIVI](https://github.com/f-io/LIVI).

## Supported versions

Only the latest release (currently none: development builds only) receives fixes.

## What open-car promises

- It never connects to the ECU, OBD port or CAN bus, and never controls the vehicle.
- It never bypasses Apple's authentication and keeps no secret keys in the repository.
- It collects no personal data and uploads nothing; diagnostics stay on the device.

Details, risks and reviews: [docs/MASTER_SECURITY.md](docs/MASTER_SECURITY.md).
