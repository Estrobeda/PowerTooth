# Firmware host-link

The canonical protocol description is `host/device-link/readme.md`. This component owns only framing and command dispatch. Persistent address operations live in `controller-registry`, while PC-state and pairing indication live in `power-control`.
