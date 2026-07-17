# PowerTooth plan

PowerTooth replaces the Arduino prototype with ESP-IDF firmware and a Linux BlueZ host bridge. The plan remains exactly ten steps.

1. **Host-link foundation — implemented.** Versioned framing separates protocol traffic from ESP-IDF logs; Rust parser tests cover every message.
2. **Controller wake and PC-state debug — controller detection verified; PC-state bench verification pending.** Firmware boots with `PAIRING`, scans for stored BLE addresses, samples power-LED state, and emits explicit wake/power logs. ESP-IDF 5.5.3 builds pass for C3 and C6; the C6 recognized stored controller `cc:b1:3f:cf:c8:7b` at RSSI -58 and safely suppressed its power pulse in bench mode.
3. **Persistent controller registry — implemented and build-verified.** ESP-IDF NVS stores a configurable number of normalized addresses; add/remove/list/reset are idempotent.
4. **BlueZ reconciliation — implemented, Linux verification pending.** The host classifies paired gamepads and sends minimal registry differences at startup and periodically.
5. **Host-driven pairing — implemented, Linux verification pending.** A firmware `PAIR` event starts bounded BlueZ discovery and pairs the first eligible gamepad.
6. **Case controls and indication — implemented, bench verification pending.** Short press passes through after debounce; long press requests pairing; the case LED flashes while pairing and otherwise mirrors motherboard state.
7. **Wake safety policy — implemented, calibration pending.** Stable power-LED sampling, a 30-second cooldown, and event coalescing gate power pulses.
8. **Reconnect and recovery — implemented.** The host retries USB connections, reconciles after reconnect, tolerates console logs, and rejects malformed protocol messages.
9. **Packaging and installation — implemented.** Reproducible build commands, a locked Rust dependency graph, a systemd unit, udev rule, and installation documentation are included.
10. **Hardware validation and release — blocked on physical Linux/ESP32/PC bench.** The validation checklist covers electrical isolation, LED calibration, pairing, reconnects, multiple controllers, and release evidence.

Current work: step 10 hardware validation. The connected board was identified read-only as ESP32-C6, and target-correct C3/C6 firmware builds pass. Flashing and GPIO tests remain blocked until the attached circuitry and configured polarities are confirmed.
