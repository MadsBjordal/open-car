# Installing on Linux (Raspberry Pi or PC)

open-car builds on LIVI, so there are two parts: LIVI (the projection engine and screen) and the
`opencar` tool.

> Nothing here has been tested in a car by this project yet. Try it on a desk first.

## 1. The computer

- A Raspberry Pi (Pi 3/4/5, CM4/CM5, Zero 2 W) with **Raspberry Pi OS based on Debian 13
  (Trixie)**, or any x86_64 Linux PC (Debian 13, Ubuntu 26.04 or Fedora 44 are tested by LIVI).
- A screen, a USB port for the phone; for wireless: Bluetooth and Wi-Fi.
- For CarPlay: an Apple MFi coprocessor on I2C, or a LIVI Link.

## 2. LIVI (its own installer)

```bash
curl -fL -o install.sh https://raw.githubusercontent.com/f-io/LIVI/main/scripts/install/install.sh
chmod +x install.sh
./install.sh
```

It installs `~/LIVI/LIVI.AppImage`, the packages LIVI needs and its start-at-boot entry.
Read the script before running it: it uses `sudo`. open-car tested with: LIVI v10.1.0 (not yet in
a car).

## 3. opencar

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
git clone https://github.com/MadsBjordal/open-car && cd open-car
cargo build --release
sudo install -m 755 target/release/opencar /usr/local/bin/opencar
```

(Prebuilt packages are on the roadmap.)

## 4. Check and configure

```bash
opencar check                              # what is ready, what is missing
opencar compat search raspberry            # find your device
opencar setup --profile raspberry-pi-5     # say which device this is
opencar setup --mfi-bus 1                  # only if an MFi chip is wired to I2C bus 1
opencar livi configure --dry-run           # see what would change in LIVI's settings
opencar livi configure                     # write it (stop LIVI first); old copy kept as .bak
```

## Remove

`sudo rm /usr/local/bin/opencar` and `rm -r ~/.config/open-car`. LIVI is removed as LIVI's
README describes (`sudo systemctl disable livi-kiosk.service`, delete `~/LIVI`).
