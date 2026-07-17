# Linux installation

Prerequisites on Debian/Ubuntu are Rust, `pkg-config`, `libdbus-1-dev`, BlueZ, and a user with permission to access the ESP32 serial device.

Build with the checked dependency lockfile:

```sh
cargo test --locked --manifest-path host/Cargo.toml
cargo build --release --locked --manifest-path host/Cargo.toml
sudo install -m 0755 host/target/release/powertooth-host /usr/local/bin/powertooth-host
sudo install -m 0644 packaging/powertooth.service /etc/systemd/system/powertooth.service
sudo install -m 0644 packaging/99-powertooth.rules /etc/udev/rules.d/99-powertooth.rules
sudo udevadm control --reload-rules
sudo systemctl daemon-reload
sudo systemctl enable --now powertooth.service
```

The sample udev rule matches Espressif's USB vendor ID. Narrow it with the board's product ID or serial number if multiple Espressif devices are connected. Adjust `--device` in the unit if the stable symlink is not created.

Inspect operation with:

```sh
systemctl status powertooth.service
journalctl -u powertooth.service -f
```

