#!/bin/sh
set -eu

SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
PROJECT_DIR=$(CDPATH= cd -- "$SCRIPT_DIR/.." && pwd)
OUTPUT_DIR="$SCRIPT_DIR/bin/build"

if [ "$(uname -s)" != "Linux" ]; then
    echo "This script must run on Linux: a normal macOS Rust build cannot run on Linux." >&2
    echo "Copy the source to Linux, or run this script in a Linux build environment." >&2
    exit 1
fi

FEATURE_ARGS=""
SUFFIX=""
case "${1:-}" in
    "") ;;
    --debug)
        FEATURE_ARGS="--features debug-logging"
        SUFFIX="-debug"
        ;;
    *)
        echo "Usage: sh host/package-linux.sh [--debug]" >&2
        exit 2
        ;;
esac

if ! command -v cargo >/dev/null 2>&1; then
    echo "Rust/Cargo is required to build the Linux host." >&2
    exit 1
fi
if ! command -v zip >/dev/null 2>&1; then
    echo "The zip command is required to create the release archive." >&2
    exit 1
fi

ARCH=$(uname -m)
BUNDLE_NAME="powertooth-linux-${ARCH}${SUFFIX}"
TEMP_DIR=$(mktemp -d)
trap 'rm -rf "$TEMP_DIR"' EXIT HUP INT TERM
STAGE_DIR="$TEMP_DIR/$BUNDLE_NAME"

echo "Building PowerTooth for Linux..."
# FEATURE_ARGS intentionally expands to zero or two command arguments.
# shellcheck disable=SC2086
cargo test --locked --manifest-path "$SCRIPT_DIR/Cargo.toml" $FEATURE_ARGS
# shellcheck disable=SC2086
cargo build --release --locked --manifest-path "$SCRIPT_DIR/Cargo.toml" $FEATURE_ARGS

mkdir -p "$STAGE_DIR" "$OUTPUT_DIR"
install -m 0755 "$SCRIPT_DIR/target/release/powertooth-host" "$STAGE_DIR/powertooth-host"
install -m 0644 "$PROJECT_DIR/packaging/powertooth.service" "$STAGE_DIR/powertooth.service"
install -m 0644 "$PROJECT_DIR/packaging/99-powertooth.rules" "$STAGE_DIR/99-powertooth.rules"
install -m 0644 "$PROJECT_DIR/packaging/powertooth.logrotate" "$STAGE_DIR/powertooth.logrotate"
install -m 0755 "$PROJECT_DIR/packaging/install.sh" "$STAGE_DIR/install.sh"
install -m 0644 "$PROJECT_DIR/AI_AGENT_NOTICE.md" "$STAGE_DIR/AI_AGENT_NOTICE.md"

rm -f "$OUTPUT_DIR/$BUNDLE_NAME.zip"
(cd "$TEMP_DIR" && zip -qr "$OUTPUT_DIR/$BUNDLE_NAME.zip" "$BUNDLE_NAME")

echo "Created $OUTPUT_DIR/$BUNDLE_NAME.zip"
echo "Copy it to Linux, unzip it, enter $BUNDLE_NAME, and run: sh ./install.sh"
