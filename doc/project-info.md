# Project Info

Project: serial-mcp-server
Language: Rust 2021
Package: `serial-mcp-server` v0.3.0
Repository: https://github.com/adancurusul/serial-mcp-server

## Shape

The project exposes a serial communication runtime through:

- CLI commands in `src/cli.rs` and argument/config types in `src/config.rs`.
- MCP tools in `src/tools/serial_handler.rs` and request/response types in `src/tools/types.rs`.
- Serial connection primitives in `src/serial/`.
- Macro automation in `src/automation/`.

## Current Task

Task: M1 serial capture window
Branch: `task/serial-capture-window`
Worktree: `serial-mcp-server-task-serial-capture-window`

Implement a backward-compatible blocking capture window for CLI and MCP reads.
Existing single-read timeout behavior must remain unchanged when `duration_ms`
is absent.

## Commands

- Format: `cargo fmt --check`
- Test: `cargo test --locked`
- Help smoke: `cargo run --locked -- read --help`

