# PowerTooth Linux host

The Rust host bridge reads paired devices from BlueZ, classifies gamepads using
BlueZ's `input-gaming` icon or a combination of the standard HID service UUID
and a controller-like device name, and synchronizes their Bluetooth addresses
to the ESP32. No controller address is compiled into either program.

At startup, the host compares BlueZ's paired gamepads with the ESP32 registry
using `LIST`, sends only the necessary `ADD` and `REMOVE` commands, and verifies
that both sides converge. It polls BlueZ every ten seconds by default, but does
not use the serial link unless the paired set changes. A `PT/1 PAIR` event from
the ESP32 starts a thirty-second BlueZ discovery and pairing window, followed by
another registry synchronization.

The process needs permission to access BlueZ over the system D-Bus and to open the ESP32 serial device.

## Command line

After installation, inspect all runtime options with:

```sh
powertooth help
```

To clear every controller address stored in the ESP32 and then exit:

```sh
powertooth reset
```

This maintenance command talks directly to the ESP32 and does not require a
working BlueZ connection. Normal synchronization never clears the entire
registry; it applies only the required additions and removals.

Useful development commands from the repository root are:

```sh
cargo run --manifest-path host/Cargo.toml --bin powertooth -- --list-bluez
cargo run --manifest-path host/Cargo.toml --bin powertooth -- --device /dev/ttyACM0 --baud 115200
```

## Release bundle

From the repository root, create an x86-64 Bazzite installer archive in a
disposable Fedora Podman container with:

```sh
sh ./publish.sh
```

Use `sh ./publish.sh --debug` to include non-protocol ESP32 firmware output in
the host log, or add `--arch arm64` for an ARM64 destination. Normal builds
already log every outbound protocol command and inbound protocol response or
event. The archive is written to `host/bin/build/`. Its bundled `install.sh`
installs and starts the systemd service and configures
`/var/log/powertooth/host.log` without layering build dependencies on Bazzite.

The host keeps DTR and RTS inactive, waits one second after opening USB, and
retries `HELLO` three times without reopening the port by default. These values,
the serial device and baud rate, the BlueZ polling interval, and the pairing
timeout are command-line options.

## Build-time defaults

Edit `host/build-defaults.conf` before compiling to change the defaults embedded
in the executable:

```text
PROTOCOL_PREFIX=PT/1
DEVICE=/dev/ttyACM0
BAUD=115200
CONNECT_DELAY_MS=1000
HANDSHAKE_ATTEMPTS=3
INTERVAL_SECONDS=10
PAIR_TIMEOUT_SECONDS=30
```

These are defaults, not fixed settings: every value can still be changed when
the installed program is started, for example
`powertooth --handshake-attempts 5`. For automated builds, prefix a setting with
`POWERTOOTH_DEFAULT_` to override the file, such as
`POWERTOOTH_DEFAULT_INTERVAL_SECONDS=20 sh ./publish.sh`.

`powertooth -V` reports the release tag embedded when the host is compiled.
Release builds receive it from the tagged pipeline. Local builds made exactly at
a Git tag use that tag; builds without a tag report `0.0.0-default`. Set
`POWERTOOTH_VERSION` explicitly to override version discovery.
