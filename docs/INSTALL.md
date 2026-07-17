# Linux installation

## Release bundle (recommended)

Build the archive on Linux with the same CPU architecture as the destination:

```sh
sh host/package-linux.sh
```

For a diagnostic build that records every command and response crossing the
ESP32 host link:

```sh
sh host/package-linux.sh --debug
```

Both commands create a ZIP file under `host/bin/build/`. Copy that archive to
the destination, then run:

```sh
unzip powertooth-linux-*.zip
cd powertooth-linux-*
sh ./install.sh
```

The installer requests root access through `sudo`, installs BlueZ and D-Bus when
needed on Debian/Ubuntu, Fedora, or Arch Linux, and installs and starts the host
service. Once the ESP32 appears as `/dev/powertooth`, a long case-button press
sends the pairing request to the running host.

The persistent host log is:

```text
/var/log/powertooth/host.log
```

Follow it with:

```sh
tail -f /var/log/powertooth/host.log
```

Logs rotate weekly, retain eight compressed rotations, and can also be inspected
through `journalctl -u powertooth.service` for systemd lifecycle messages.

## Manual installation

Prerequisites on Debian/Ubuntu are Rust, `pkg-config`, `libdbus-1-dev`, BlueZ, and a user with permission to access the ESP32 serial device.

Build with the checked dependency lockfile:

```sh
cargo test --locked --manifest-path host/Cargo.toml
cargo build --release --locked --manifest-path host/Cargo.toml
sudo install -m 0755 host/target/release/powertooth-host /usr/local/bin/powertooth-host
sudo install -m 0644 packaging/powertooth.service /etc/systemd/system/powertooth.service
sudo install -m 0644 packaging/99-powertooth.rules /etc/udev/rules.d/99-powertooth.rules
sudo install -m 0644 packaging/powertooth.logrotate /etc/logrotate.d/powertooth
sudo install -d -m 0755 /var/log/powertooth
sudo touch /var/log/powertooth/host.log
sudo udevadm control --reload-rules
sudo systemctl daemon-reload
sudo systemctl enable --now powertooth.service
```

The sample udev rule matches Espressif's USB vendor ID. Narrow it with the board's product ID or serial number if multiple Espressif devices are connected. Adjust `--device` in the unit if the stable symlink is not created.

Inspect operation with:

```sh
systemctl status powertooth.service
journalctl -u powertooth.service -f
tail -f /var/log/powertooth/host.log
```
