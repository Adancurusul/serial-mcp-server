# Changelog

## 0.3.0 - 2026-06-24

### Added

- Added v0.3 JSON Macro DSL packs with strict validation for macros and assemblies.
- Added CLI macro commands: `macro validate`, `macro list`, `macro plan`, and `macro run`.
- Added macro executor support for `send`, `delay`, `expect contains`, and `expect equals`.
- Added explicit simulation mode for no-hardware macro execution tests and CLI smoke checks.
- Added MCP macro tools: `macro_load`, `macro_list`, `macro_unload`, `macro_plan`, `macro_run`, and `macro_run_inline`.
- Added runtime-only MCP macro registry. Loaded packs are cleared on server restart.
- Added `examples/macros/ping.json` as a minimal valid macro pack.

### Changed

- Updated package version to `0.3.0`.
- Updated README, README_ZH, and serial-debug skill docs to describe CLI, skill, and MCP macro usage paths.

### Not Included

- No independent Quick API. Quick-style operations should be represented as macros.
- No persistent macro library managed by the server.
- No RTS/DTR macro steps and no general scripting features.

## 0.2.0 - 2026-06-24

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
