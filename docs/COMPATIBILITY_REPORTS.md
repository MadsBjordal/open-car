# Reporting a head unit or adapter

1. On the device: `opencar check` and `opencar diag -o report.json`. Read `report.json` before
   sharing it: it should hold no serial numbers, addresses or personal data; tell us if it does.
2. Open a **Compatibility report** issue (template on GitHub) with: maker, model, hardware
   revision, firmware, the car(s) it came in, how you installed, and what you tested (wired/
   wireless Android Auto, CarPlay, audio, mic, calls, touch, knobs, steering-wheel buttons), with
   the phone models and their OS versions, plus photos of the unit's labels.
3. A maintainer turns it into a profile in `compat/profiles/` with your test as `[[evidence]]`.
   Only then can a status be `experimental` or `verified`. `opencar compat validate` refuses a
   `verified` profile without evidence and a CarPlay claim without an authentication route.

Never include VINs, phone numbers, contacts, MFi keys or firmware files.
