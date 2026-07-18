# Hardware validation and release checklist

Nothing in this checklist is marked passed by source compilation alone.

## Electrical, disconnected from motherboard

- [x] Use a bench build that suppresses physical power output during BLE recognition. Verified on ESP32-C6, 2026-07-17; the temporary suppression code was removed after the test.
- [x] With the selected configuration, confirm the power-switch GPIO is inactive at reset and produces one configured-duration pulse for a short case-button press.
- [x] Confirm holding the case button for two seconds flashes the case LED and does not pulse GPIO 5.
- [x] Inject low/high into the accessible GPIO 4 test point and observe `POWER OFF` / `POWER ON`.
- [x] Confirm GPIO 6 flashes at 1 Hz while pairing and mirrors GPIO 4 afterward.
- [x] Verify isolation-stage polarity and voltage with a multimeter.

## Linux and BlueZ

- [x] `--list-bluez` prints paired gamepads and excludes unrelated Bluetooth devices.
- [x] Empty ESP32 registry is populated after host connection.
- [x] Removing a gamepad in BlueZ removes it from ESP32 NVS on reconciliation.
- [x] Long-press pairing discovers and pairs the intended controller within the timeout.
- [x] Restarting BlueZ, unplugging USB, and resetting ESP32 all recover without manual registry reset.

## ESP32-only known-address test

- [x] Build temporary bench firmware with physical power output suppressed and flash the intended board. ESP32-C6 verified 2026-07-17; production code was restored afterward.
- [x] Send `PT/1 ADD cc:b1:3f:cf:c8:7b` and receive `PT/1 OK`.
- [x] Send `PT/1 LIST` and receive the same `PT/1 DEVICE` address followed by `PT/1 END`.
- [x] Reset the ESP32 and repeat `LIST` to prove NVS persistence.
- [x] Put that device into an advertising state and observe `Known device recognized: cc:b1:3f:cf:c8:7b RSSI=-58`.
- [x] Confirm the corresponding wake attempt says `Power pulse suppressed (bench mode): stored controller detected`.

## Motherboard bench

- [ ] Determine PLED voltage, polarity, steady-on behavior, and sleep blinking behavior.
- [ ] Confirm the case LED preserves normal PC indication outside pairing mode.
- [ ] Confirm controller detection never pulses `PWR_SW` while the PC is already on.
- [ ] Confirm a stored controller wakes the PC exactly once while it is off.
- [ ] Confirm cooldown prevents repeated pulses for at least 30 seconds.
- [ ] Validate at least two controllers and record controller model, transport, and observed address behavior.

## Release evidence

- [ ] Record ESP-IDF version, board model/revision, BlueZ version, motherboard, schematic, and calibrated threshold.
- [ ] Attach successful firmware build output and Linux `cargo test --locked` output.
- [ ] Tag a release only after every applicable checkbox above passes.
