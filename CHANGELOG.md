# Changelog

All notable changes to this project will be documented in this file.

The format is based on Keep a Changelog, and this project adheres to Semantic Versioning.

## [0.1.0] - 2026-04-13

### Added
- Audio domain implementation with sink/source listing, volume, and mute controls.
- Network CLI actions for WiFi scan/connect/disconnect and Ethernet connect/disconnect.
- Proc watch support to wait for a PID to exit.
- NetworkManager domain now maintains a persistent connection with periodic status refresh and event publishing.
- Network master power endpoint and status field for global NM enable/disable.
- Bluetooth pairing features: pair, trust, remove, alias rename, discoverable, pairable, and auth agent.
- Bluetooth Battery1 support for device battery percentage.
- Implemented sysmon `--watch` in the CLI by consuming the SSE event stream.
- Wired sysmon and brightness HTTP endpoints in the daemon.

### Changed
- Disk endpoints now wired to real UDisks2 operations.
- Network status now reports mode (station/ap/unknown) and a connected AP always wins WiFi dedupe.
- Network power now uses master networking switch (removed per-wifi power control).
- Network events now include `mode_changed`.
- Improved SSE serialization handling by logging failures instead of emitting empty events.
- Aligned the systemd unit with the `~/.local/bin/crawlds-daemon` install path.
- Updated `wl-clipboard-rs` to 0.9.3 to address future Rust incompatibilities.

### Documentation
- Updated IPC docs with new endpoints and network mode info.