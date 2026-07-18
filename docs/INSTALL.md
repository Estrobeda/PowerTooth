# Linux installation

## Release bundle (recommended)

Build the x86-64 Bazzite archive from Linux or macOS using the disposable Fedora
Podman builder:

```sh
sh ./publish.sh
```

The host always logs the command flow crossing the ESP32 host link. For a
diagnostic build that additionally records the ESP32's own firmware log
output:

```sh
sh ./publish.sh --debug
```

For an ARM64 destination, add `--arch arm64`. The default is `--arch amd64`.
Both commands create a ZIP file under `host/bin/build/`. Copy that archive to
the Bazzite destination, then run:

```sh
unzip powertooth-linux-*.zip
cd powertooth-linux-*
sh ./install.sh
```

The installer requests root access through `sudo`, validates the artifact before
changing the system, and installs and starts the host service. It uses Bazzite's
existing BlueZ and system D-Bus instead of layering development packages with
`rpm-ostree`. Once the ESP32 appears as `/dev/powertooth`, a long case-button
press sends the pairing request to the running host.

Podman must be running before publishing. On macOS:

```sh
podman machine init
podman machine start
```

The publisher limits Cargo to one compilation job to fit comfortably inside an
emulated Podman VM. If an x86-64 build still ends with `signal: 9, SIGKILL`, give
the Podman machine more memory and retry:

```sh
podman machine stop
podman machine set --memory 6144 --cpus 4
podman machine start
sh ./publish.sh --debug --arch amd64
```

The Fedora image can be overridden when necessary:

```sh
POWERTOOTH_BUILD_IMAGE=registry.fedoraproject.org/fedora:latest sh ./publish.sh
```

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

## Removal

The bundle ships an uninstaller. From the unzipped bundle directory:

```sh
sh ./uninstall.sh
```

It stops and disables the service, then deletes the binary, the systemd unit,
the udev rule, the logrotate configuration, and the entire log directory
`/var/log/powertooth`. BlueZ and any controllers paired with the PC are left
untouched.

## Manual installation

Prerequisites on Debian/Ubuntu are Rust, `pkg-config`, `libdbus-1-dev`, BlueZ, and a user with permission to access the ESP32 serial device.

Build with the checked dependency lockfile:

```sh
cargo test --locked --manifest-path host/Cargo.toml
cargo build --release --locked --manifest-path host/Cargo.toml
sudo install -m 0755 host/target/release/powertooth /usr/local/bin/powertooth
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
It also tells ModemManager to ignore the ESP32 serial interface. The host waits
one second after opening USB by default so boards that reset on open can finish
booting; adjust this with `--connect-delay-ms` if necessary.

Inspect operation with:

```sh
systemctl status powertooth.service
journalctl -u powertooth.service -f
tail -f /var/log/powertooth/host.log
```
