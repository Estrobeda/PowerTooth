#!/bin/sh
set -eu

if [ "$(id -u)" -ne 0 ]; then
    if command -v sudo >/dev/null 2>&1; then
        exec sudo sh "$0" "$@"
    fi
    echo "PowerTooth installation requires root privileges." >&2
    exit 1
fi

SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)

for required_file in powertooth-host powertooth.service 99-powertooth.rules powertooth.logrotate; do
    if [ ! -f "$SCRIPT_DIR/$required_file" ]; then
        echo "Missing bundle file: $required_file" >&2
        exit 1
    fi
done

if ! command -v systemctl >/dev/null 2>&1; then
    echo "PowerTooth requires a systemd-based Linux distribution." >&2
    exit 1
fi

install_dependencies() {
    if command -v bluetoothctl >/dev/null 2>&1; then
        return
    fi

    echo "Installing BlueZ and D-Bus runtime dependencies..."
    if command -v apt-get >/dev/null 2>&1; then
        apt-get update
        DEBIAN_FRONTEND=noninteractive apt-get install -y bluez dbus libdbus-1-3
    elif command -v dnf >/dev/null 2>&1; then
        dnf install -y bluez dbus
    elif command -v pacman >/dev/null 2>&1; then
        pacman -Sy --needed --noconfirm bluez bluez-utils dbus
    else
        echo "Install BlueZ and D-Bus, then rerun this installer." >&2
        exit 1
    fi
}

install_dependencies

echo "Installing PowerTooth host bridge..."
install -m 0755 "$SCRIPT_DIR/powertooth-host" /usr/local/bin/powertooth-host
install -m 0644 "$SCRIPT_DIR/powertooth.service" /etc/systemd/system/powertooth.service
install -m 0644 "$SCRIPT_DIR/99-powertooth.rules" /etc/udev/rules.d/99-powertooth.rules
install -m 0644 "$SCRIPT_DIR/powertooth.logrotate" /etc/logrotate.d/powertooth
install -d -m 0755 /var/log/powertooth
touch /var/log/powertooth/host.log
chmod 0644 /var/log/powertooth/host.log

udevadm control --reload-rules
udevadm trigger --subsystem-match=tty
systemctl daemon-reload
systemctl enable --now bluetooth.service
systemctl enable --now powertooth.service

echo
echo "PowerTooth is installed and running."
echo "Connect the ESP32 and check: systemctl status powertooth.service"
echo "Follow the host log with:       tail -f /var/log/powertooth/host.log"
echo "Pair a controller by holding the configured case button for two seconds."
