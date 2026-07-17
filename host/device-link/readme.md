# Host-link protocol

UTF-8 command lines are terminated by `\n` and begin with `PT/1 `. This framing allows the bridge to ignore ESP-IDF console logs sharing the USB serial connection. Bluetooth addresses are lowercase canonical strings (`aa:bb:cc:dd:ee:ff`). Every mutating command receives exactly one `OK` or `ERR <reason>` response.

Host to ESP32:

- `PT/1 HELLO` — select protocol version 1.
- `PT/1 ADD <address>` — idempotently add a gamepad address.
- `PT/1 REMOVE <address>` — idempotently remove an address.
- `PT/1 LIST` — return zero or more `DEVICE` lines followed by `END`.
- `PT/1 RESET` — clear the registry.
- `PT/1 SYNC` — mark reconciliation complete and stop pairing indication.
- `PT/1 POWER?` — return `POWER ON` or `POWER OFF` from the motherboard LED sense input.

ESP32 to host:

- `PT/1 PAIR` — request host-owned BlueZ discovery and pairing.
- `PT/1 OK` — command completed.
- `PT/1 ERR <reason>` — command rejected.
