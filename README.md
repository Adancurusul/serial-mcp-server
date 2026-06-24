# Serial MCP Server

[![Rust](https://img.shields.io/badge/rust-1.74+-orange.svg)](https://rust-lang.org)
[![RMCP](https://img.shields.io/badge/RMCP-0.3.2-blue.svg)](https://github.com/modelcontextprotocol/rust-sdk)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)

`serial-mcp-server` provides serial port access for AI workflows in two forms:

- MCP stdio server for clients that support MCP tools.
- Scriptable CLI for direct use, CI, and agent skills without MCP setup.

Current release target: `0.2.0`.

Language versions: [English](README.md) | [Chinese](README_ZH.md)

## Requirements

- Rust 1.74 or newer.
- A serial device or USB-to-serial adapter when running hardware operations.
- Device drivers and OS permissions for the selected serial port.

## Install From Source

```bash
git clone https://github.com/adancurusul/serial-mcp-server.git
cd serial-mcp-server
cargo build --release
```

The binary is built at:

```bash
target/release/serial-mcp-server
```

To install the CLI onto your `PATH` from a checkout:

```bash
cargo install --path . --locked
```

## CLI Usage

Use the CLI when you want direct serial operations without an MCP client.

```bash
serial-mcp-server --help
serial-mcp-server list-ports --json
serial-mcp-server probe --port <port> --baud 115200 --json
serial-mcp-server write --port <port> --baud 115200 --data H --read --timeout-ms 1000 --json
serial-mcp-server read --port <port> --baud 115200 --timeout-ms 1000 --json
serial-mcp-server set-control-lines --port <port> --rts high --dtr low --json
```

Configuration commands:

```bash
serial-mcp-server generate-config
serial-mcp-server validate-config --config serial-mcp.toml
serial-mcp-server show-config --config serial-mcp.toml
```

CLI output rules:

- stdout is reserved for command data and JSON.
- stderr is reserved for diagnostics.
- `--json` output should be parseable by tools such as `jq`.
- Nonzero exit codes indicate command failure.

Supported CLI data formats are `utf8`, `hex`, and `base64`. Use `hex` or `base64` for binary payloads.

## MCP Usage

Use MCP when your client supports MCP tools and you want a long-running stdio server.

Recommended server command:

```bash
serial-mcp-server serve
```

No-subcommand startup is retained as a compatibility path for existing MCP setups, but new configurations should use `serve`.

Claude Desktop example for macOS/Linux:

```json
{
  "mcpServers": {
    "serial": {
      "command": "/path/to/serial-mcp-server/target/release/serial-mcp-server",
      "args": ["serve"],
      "env": {
        "RUST_LOG": "info"
      }
    }
  }
}
```

Windows example:

```json
{
  "mcpServers": {
    "serial": {
      "command": "C:\\path\\to\\serial-mcp-server\\target\\release\\serial-mcp-server.exe",
      "args": ["serve"],
      "env": {
        "RUST_LOG": "info"
      }
    }
  }
}
```

## MCP Tools

| Tool | Purpose |
| --- | --- |
| `list_ports` | Discover available serial ports. |
| `open` | Open a serial connection. |
| `write` | Write UTF-8, hex, or base64 data to an open connection. |
| `read` | Read data from an open connection with timeout handling. |
| `close` | Close an open connection. |
| `set_control_lines` | Set RTS and/or DTR on an open connection. |

## Agent Skill

The repository includes a Claude Code and Codex compatible skill at:

```text
skills/serial-debug/
```

The skill is CLI-first and documents MCP as an optional configured path. It is intended for agents that need to list ports, run serial smoke tests, control RTS/DTR, or troubleshoot UART/USB-serial devices.

For local development, copy the skill folder into the agent skill roots:

```bash
mkdir -p ~/.codex/skills ~/.claude/skills ~/.agents/skills
cp -R skills/serial-debug ~/.codex/skills/
cp -R skills/serial-debug ~/.claude/skills/
cp -R skills/serial-debug ~/.agents/skills/
```

Tested explicit triggers:

```text
Codex: Use $serial-debug
Claude Code: /serial-debug
```

Claude Code `--bare` mode did not resolve `/serial-debug` in local testing; use normal Claude Code print or interactive mode for skill-trigger smoke tests.

## Hardware Safety

Serial commands can affect real hardware.

- Confirm the selected port from `serial-mcp-server list-ports --json`.
- Confirm voltage levels before connecting an adapter to a target board.
- Confirm baud rate, data bits, parity, stop bits, and flow control before writing.
- Treat RTS and DTR carefully. Many boards wire those lines to reset or boot mode.
- Do not claim a write/read or RTS/DTR validation passed unless the command ran against a connected device.

## STM32 Demo

The STM32 demo is under:

```text
examples/STM32_demo/
```

It provides firmware for an interactive serial command interface. See [examples/STM32_demo/README.md](examples/STM32_demo/README.md) for wiring, firmware commands, MCP usage, and CLI smoke commands.

## Quality Gates

Release work uses the checked-in `Cargo.lock` and these gates:

```bash
cargo fmt --all -- --check
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo test --locked --all-targets --all-features
cargo doc --locked --all-features --no-deps
```

CLI smoke:

```bash
cargo run --locked -- --help
cargo run --locked -- list-ports --json
cargo run --locked -- write --help
cargo run --locked -- set-control-lines --help
```

## License

This project is licensed under the MIT License. See [LICENSE](LICENSE).
