#!/bin/sh
set -eu

if [ "$(id -u)" -ne 0 ]; then
    if command -v sudo >/dev/null 2>&1; then
        exec sudo sh "$0" "$@"
    fi
    echo "PowerTooth removal requires root privileges." >&2
    exit 1
fi

if ! command -v systemctl >/dev/null 2>&1; then
    echo "PowerTooth installs only on systemd-based Linux distributions; nothing to remove." >&2
    exit 1
fi

echo "Removing PowerTooth host bridge..."
systemctl disable --now powertooth.service 2>/dev/null || true
systemctl reset-failed powertooth.service 2>/dev/null || true

rm -f /usr/local/bin/powertooth \
    /usr/local/bin/powertooth-host \
    /etc/systemd/system/powertooth.service \
    /etc/udev/rules.d/99-powertooth.rules \
    /etc/logrotate.d/powertooth
rm -rf /var/log/powertooth

systemctl daemon-reload
if command -v udevadm >/dev/null 2>&1; then
    udevadm control --reload-rules
    # Re-evaluate tty devices so a connected ESP32 loses the /dev/powertooth
    # symlink immediately instead of on its next replug.
    udevadm trigger --subsystem-match=tty
fi

echo
echo "PowerTooth has been removed, including its logs."
echo "The Bluetooth service and any paired controllers were left untouched."
