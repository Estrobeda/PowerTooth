#!/bin/sh
set -eu

PROJECT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
BUILD_IMAGE=${POWERTOOTH_BUILD_IMAGE:-registry.fedoraproject.org/fedora:latest}
TARGET_ARCH=amd64
DEBUG_BUILD=false

while [ "$#" -gt 0 ]; do
    case "$1" in
        --debug)
            DEBUG_BUILD=true
            ;;
        --arch)
            shift
            if [ "$#" -eq 0 ]; then
                echo "--arch requires amd64 or arm64" >&2
                exit 2
            fi
            TARGET_ARCH=$1
            ;;
        --help|-h)
            echo "Usage: sh ./publish.sh [--debug] [--arch amd64|arm64]"
            exit 0
            ;;
        *)
            echo "Unknown option: $1" >&2
            echo "Usage: sh ./publish.sh [--debug] [--arch amd64|arm64]" >&2
            exit 2
            ;;
    esac
    shift
done

case "$TARGET_ARCH" in
    amd64)
        PLATFORM=linux/amd64
        RUST_ARCH=x86_64
        ;;
    arm64)
        PLATFORM=linux/arm64
        RUST_ARCH=aarch64
        ;;
    *)
        echo "Unsupported architecture: $TARGET_ARCH (use amd64 or arm64)" >&2
        exit 2
        ;;
esac

if ! command -v podman >/dev/null 2>&1; then
    echo "Podman is required. On macOS, install Podman and run: podman machine init && podman machine start" >&2
    exit 1
fi
if ! podman info >/dev/null 2>&1; then
    echo "Podman is installed but not running. On macOS, run: podman machine start" >&2
    exit 1
fi

if [ "$DEBUG_BUILD" = true ]; then
    PACKAGE_ARG=--debug
    SUFFIX=-debug
else
    PACKAGE_ARG=
    SUFFIX=
fi

BUNDLE_NAME="powertooth-linux-${RUST_ARCH}${SUFFIX}.zip"

echo "Building $BUNDLE_NAME in $BUILD_IMAGE for $PLATFORM..."
podman run --rm \
    --platform "$PLATFORM" \
    --security-opt label=disable \
    -e CARGO_BUILD_JOBS=1 \
    -e CARGO_PROFILE_DEV_DEBUG=0 \
    -e CARGO_TARGET_DIR="/workspace/host/target/container-${RUST_ARCH}" \
    -e POWERTOOTH_PACKAGE_ARG="$PACKAGE_ARG" \
    -e POWERTOOTH_DEFAULT_PROTOCOL_PREFIX="${POWERTOOTH_DEFAULT_PROTOCOL_PREFIX:-}" \
    -e POWERTOOTH_DEFAULT_DEVICE="${POWERTOOTH_DEFAULT_DEVICE:-}" \
    -e POWERTOOTH_DEFAULT_BAUD="${POWERTOOTH_DEFAULT_BAUD:-}" \
    -e POWERTOOTH_DEFAULT_CONNECT_DELAY_MS="${POWERTOOTH_DEFAULT_CONNECT_DELAY_MS:-}" \
    -e POWERTOOTH_DEFAULT_HANDSHAKE_ATTEMPTS="${POWERTOOTH_DEFAULT_HANDSHAKE_ATTEMPTS:-}" \
    -e POWERTOOTH_DEFAULT_INTERVAL_SECONDS="${POWERTOOTH_DEFAULT_INTERVAL_SECONDS:-}" \
    -e POWERTOOTH_DEFAULT_PAIR_TIMEOUT_SECONDS="${POWERTOOTH_DEFAULT_PAIR_TIMEOUT_SECONDS:-}" \
    -v "$PROJECT_DIR:/workspace" \
    -w /workspace \
    "$BUILD_IMAGE" \
    sh -lc '
        set -eu
        dnf install -y rust cargo gcc dbus-devel pkgconf-pkg-config zip
        sh host/package-linux.sh $POWERTOOTH_PACKAGE_ARG
    '

echo "Built: $PROJECT_DIR/host/bin/build/$BUNDLE_NAME"
echo "Copy the ZIP to Bazzite, unzip it, enter the directory, and run: sh ./install.sh"
