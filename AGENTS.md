# PowerTooth working agreement

- Keep the roadmap in `PLAN.md` at exactly ten steps unless the user explicitly approves changing the count.
- Work in small, independently testable changes.
- Keep the ESP32 firmware simple. Linux/BlueZ owns discovery, pairing, removal policy, and gamepad classification.
- The ESP32 owns power-button output, motherboard power-LED sensing, pairing indication, and a persistent allow-list of controller Bluetooth addresses.
- Treat controller addresses as runtime data synchronized by the host; never compile personal addresses into firmware.
- Keep board assumptions explicit. ESP32-C3 and ESP32-C6 builds are supported; the board detected on `/dev/cu.usbmodem101` on 2026-07-17 was an ESP32-C6. Do not treat a successful build as physical GPIO validation.
- Preserve clear bench logs and accessible test points for power-LED sense and power-button output.
