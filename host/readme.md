# PowerTooth Linux host

The Rust host bridge reads paired devices from BlueZ, selects gamepads by the standard HID service UUID or BlueZ's `input-gaming` icon, and synchronizes their Bluetooth addresses to the ESP32. No controller address is compiled into either program.

For the first slice, reconciliation runs on startup and every five seconds. It uses a deterministic full replacement; plan step 4 will consume `LIST` and send only differences.

```sh
cargo run --manifest-path host/Cargo.toml -- --list-bluez
cargo run --manifest-path host/Cargo.toml -- --device /dev/ttyACM0 --baud 115200
```

The process needs permission to access BlueZ over the system D-Bus and to open the ESP32 serial device.

Use `idf.py menuconfig` → **PowerTooth** to change board GPIOs, signal polarity, timing, BLE scan parameters, and registry capacity. Host serial device, baud rate, reconciliation interval, and pairing timeout are command-line options; run `powertooth-host --help` for the complete list.
