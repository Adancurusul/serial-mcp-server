# CLAUDE.md

This repository provides a Rust MCP stdio server and one-shot CLI for serial port communication.

## Development Rules

- Keep `main` releasable.
- Prefer small, reviewable changes with matching tests and documentation.
- Preserve CLI stdout for command data and JSON output. Send diagnostics to stderr.
- Do not claim hardware validation unless a command was run against a real connected device and produced evidence.
- Keep generated build output such as `target/` out of commits.

## Quality Gates

```bash
cargo fmt --all -- --check
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo test --locked --all-targets --all-features
cargo doc --locked --all-features --no-deps
```

## Build And Run

```bash
cargo build --release
cargo run --locked -- --help
cargo run --locked -- serve --help
cargo run --locked -- list-ports --json
```

MCP server mode:

```bash
cargo run --locked -- serve
```

No-subcommand startup remains compatible with old MCP stdio configurations, but new docs and client configs should use `serve`.

Config commands:

```bash
cargo run --locked -- generate-config
cargo run --locked -- validate-config --config path/to/config.toml
cargo run --locked -- show-config --config path/to/config.toml
```

## CLI Surface

The CLI is the preferred automation surface for scripts, CI, and skills:

```bash
serial-mcp-server list-ports --json
serial-mcp-server probe --port <port> --baud 115200 --json
serial-mcp-server write --port <port> --baud 115200 --data H --read --timeout-ms 1000 --json
serial-mcp-server read --port <port> --baud 115200 --timeout-ms 1000 --json
serial-mcp-server set-control-lines --port <port> --rts high --dtr low --json
```

stdout is data only. Diagnostics and logs go to stderr or a configured log file.

## Architecture

`serial-mcp-server` is a Rust MCP stdio server and one-shot CLI for serial port communication.

- `src/main.rs`: argument parsing, logging setup, command dispatch, MCP server startup.
- `src/config.rs`: clap arguments, subcommands, TOML config, validation.
- `src/cli.rs`: one-shot CLI command execution.
- `src/tools/serial_handler.rs`: MCP tool handler and six MCP tools.
- `src/serial/**`: serial connection wrapper, connection manager, port listing.
- `src/session/**`: session abstractions used by supporting APIs and tests.
- `skills/serial-debug/**`: Codex and Claude Code compatible skill, CLI-first with MCP optional.

## MCP Tool Model

MCP tools:

- `list_ports`
- `open`
- `write`
- `read`
- `close`
- `set_control_lines`

Connection IDs are returned by `open` and are required for `write`, `read`, `close`, and `set_control_lines`.

## Hardware Notes

RTS and DTR can be wired to reset or boot circuitry on development boards. Do not claim hardware validation unless a real command was run against a connected device and produced evidence.
