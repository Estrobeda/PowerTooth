# PowerTooth

PowerTooth wakes a PC when one of its Linux-paired Bluetooth gamepads becomes visible. It is an ESP-IDF replacement for the Arduino prototype, with controller discovery and pairing policy moved to a Linux BlueZ host bridge.

Inspired by:
https://github.com/heydemoura/esp32-bluetooth-device-turn-on-pc
https://github.com/alleras/PicoControllerWake

Somewhat inspired by
https://github.com/safijari/openpuck/tree/main

A future major change would be to fork or take inspiration from the OpenPuck project in order
to make the power-on feature a true dongle addon with wakeup functionality.
But that project would most likely also involve using or creating an open source 2.4ghz low latency protocol that can be used for gamepads now and in the future. 

Such a protocol could, for example have at least 1 known package to the hardware which would be the power-on, while button presses, gyros etc might be translated to midi if it turns out to be reasonable or some such.
Standard controller is relatively easy, abxy, start/pause, trigger L/R, bumper L/R etc but ofc such a protocol should not be limited thus abxy etc would be key (x) pressed/hold/released
joysticks, gyro and trackpad translated inputs is more complex but should similarly get a non-limiting but easy to support standard.
But this is just brainstorming on my part which may or may not see the light of day.

## Responsibilities

- **Linux host:** BlueZ discovery, gamepad classification, pairing, removals, and registry reconciliation.
- **ESP32-C3/C6:** BLE address observation, NVS allow-list, power-LED sensing, case indication, case-button handling, and isolated motherboard power-switch pulses.

Controller addresses are synchronized at runtime. There are no personal Bluetooth addresses in source code.

## GPIO assignments

These are the default ESP32-C3/C6 assignments. They can be changed under
**PowerTooth → Board pins and polarity** in `idf.py menuconfig`.

| GPIO | Direction | Purpose |
|---|---|---|
| GPIO 3 | Input, active-low | Detects the case power button. The button connects this pin to ESP32 ground. |
| GPIO 4 | Input, active-high | Senses the motherboard power-LED state through a protected or isolated 3.3 V interface. |
| GPIO 5 | Output, active-high | Drives a transistor or optocoupler that pulses the motherboard power-switch header. |
| GPIO 6 | Output, active-high | Drives the case power LED through a current-limited LED driver or transistor. It also flashes the LED during pairing. |

Do not connect a motherboard header or 5 V rail directly to an ESP32 GPIO. Use
the protection/driver stages shown below and verify motherboard header polarity
with a multimeter.

## Build

Firmware requires an ESP-IDF shell. Select the board actually connected:

```sh
cd firmware
idf.py set-target esp32c3
# or: idf.py set-target esp32c6
idf.py build
idf.py -p /dev/ttyACM0 flash monitor
```

NimBLE is the default BLE host stack. To build with the optional Bluedroid
scanner backend, recreate the configuration with the Bluedroid defaults overlay:

```sh
cd firmware
idf.py fullclean
SDKCONFIG_DEFAULTS="sdkconfig.defaults;sdkconfig.defaults.bluedroid" idf.py set-target esp32c6
SDKCONFIG_DEFAULTS="sdkconfig.defaults;sdkconfig.defaults.bluedroid" idf.py build
```

You can also select either host stack in `idf.py menuconfig` under
**Component config -> Bluetooth -> Host**. Only one host stack is compiled, and
both backends expose the same PowerTooth scanner interface.

Linux host:

```sh
cargo test --manifest-path host/Cargo.toml
cargo build --release --locked --manifest-path host/Cargo.toml
host/target/release/powertooth --list-bluez
host/target/release/powertooth --device /dev/ttyACM0
```

### Create an installable Linux bundle

The recommended publisher builds inside a disposable Fedora Podman container and
writes the transferable ZIP to `published/`. It defaults to x86-64 Bazzite:

```sh
# Normal release
sh ./publish.sh

# Release with every host/ESP32 protocol line logged
sh ./publish.sh --debug

# ARM64 Bazzite instead of the default x86-64 target
sh ./publish.sh --arch arm64
```

Podman can run this from Linux or from a macOS Podman machine; it uses emulation
when the requested Linux architecture differs from the build computer. Copy the
ZIP from `published/` to the Bazzite machine, unzip it, enter the extracted
directory, and run:

```sh
sh ./install.sh
```

The installer validates the CPU architecture and runtime libraries, avoids
`rpm-ostree` package layering on Bazzite, sets up the stable `/dev/powertooth`
device link, enables the systemd service, and writes host output to
`/var/log/powertooth/host.log`. See [Linux installation](docs/INSTALL.md) for
service and troubleshooting commands.

See [hardware wiring](docs/HARDWARE.md), [installation](docs/INSTALL.md), and the [validation checklist](docs/VALIDATION.md) before connecting a motherboard.

## AI agent notice

This project was developed with assistance from AI coding agents. See the
[AI Agent Notice](AI_AGENT_NOTICE.md) for details.

I do however try to verify and test the project thoroughly but for rappid PoC and quite frankly to have time to get the project done, AI has been invaluable.
