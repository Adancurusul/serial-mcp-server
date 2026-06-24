# Changelog

## 0.2.0 - Unreleased

### Added

- Added explicit `serve` mode for the MCP stdio server while keeping no-subcommand startup compatible.
- Added one-shot CLI commands: `list-ports`, `probe`, `write`, `read`, and `set-control-lines`.
- Added JSON output for CLI automation and agent skill workflows.
- Added `generate-config`, `validate-config`, and `show-config` subcommands alongside legacy flags.
- Added `skills/serial-debug`, a Codex and Claude Code compatible skill that uses the CLI first and MCP when configured.

### Changed

- Updated Rust MSRV documentation and package metadata to Rust 1.74.
- Updated locked dependency versions away from yanked `serialport 4.7.2` and `slab 0.4.10` after local install smoke testing.
- Updated user-facing docs to distinguish MCP, CLI, skill, and manual hardware validation boundaries.
- Refactored serial/session test modules and connection parameters to satisfy release clippy gates.

### Fixed

- Fixed CLI logging so one-shot commands keep stdout data-only by default.
- Fixed clippy warnings promoted by `-D warnings`.
- Fixed rustdoc warnings in the session documentation.

## 0.1.0

- Initial MCP stdio server with serial port discovery, open, write, read, close, and RTS/DTR control-line support.
