# Serial MCP Server

[![Rust](https://img.shields.io/badge/rust-1.74+-orange.svg)](https://rust-lang.org)
[![RMCP](https://img.shields.io/badge/RMCP-0.3.2-blue.svg)](https://github.com/modelcontextprotocol/rust-sdk)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)

`serial-mcp-server` provides serial port access for AI workflows in two forms:

- MCP stdio server for clients that support MCP tools.
- Scriptable CLI for direct use, CI, and agent skills without MCP setup.

Current release target: `0.3.0`.

Language versions: [English](README.md) | [Chinese](README_ZH.md)

## 0.3.0 Update Brief

- Added JSON Macro DSL automation for repeatable serial procedures.
- Added CLI macro commands: `macro validate`, `macro list`, `macro plan`, and `macro run`.
- Added MCP macro tools for runtime in-memory pack load/list/unload, plan, and run.
- Added explicit no-hardware simulation for macro validation, planning, and executor smoke tests.
- Kept Quick out of the public API. Quick-style use cases should be represented as macros.

## Macro Automation

Use macros when a serial workflow is more than a single read or write. Many devices require a timed sequence: send a command, wait a few milliseconds, read until a prompt or acknowledgement appears, then send the next command. A macro pack records that procedure as JSON so a human, CLI script, or AI agent can validate it, inspect the plan, simulate it without hardware, and run it against a real port when a device is attached.

Typical macro use cases:

- Boot or provisioning flows that need ordered commands and delays.
- Protocol handshakes that must wait for `OK`, `READY`, `PONG`, prompts, or other expected responses.
- Regression smoke tests where the same serial procedure should run repeatedly.
- AI-assisted debugging where the agent should review the full send/delay/expect plan before touching hardware.

The v0.3 DSL is intentionally small:

- `send`: write UTF-8, hex, or base64 bytes.
- `delay`: wait for a fixed number of milliseconds.
- `expect`: read until the response contains or equals expected bytes.
- `assembly`: compose named macros into a longer workflow.

AI agents can discover macro support from this README, from the bundled `skills/serial-debug` skill, by running `serial-mcp-server macro --help`, or through MCP tool discovery when the server is configured. Agents that do not use MCP can still use the CLI plus the skill docs.

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
serial-mcp-server read --port <port> --baud 115200 --duration-ms 5000 --initial-timeout-ms 30000 --idle-timeout-ms 1500 --json
serial-mcp-server set-control-lines --port <port> --rts high --dtr low --json
```

### Capture Window Reads

`timeout-ms` keeps its original one-read meaning: wait up to that many
milliseconds for one read operation, then return as soon as bytes are available.

Use `--duration-ms` when an AI or script needs one bounded collection window.
Capture mode keeps reading internally and returns one combined response with
`completion_reason`, `waited_ms`, `elapsed_ms`, and per-chunk metadata.

```bash
serial-mcp-server read --port <port> --baud 115200 \
  --duration-ms 5000 \
  --start-trigger first-byte \
  --initial-timeout-ms 30000 \
  --idle-timeout-ms 1500 \
  --max-bytes 8192 \
  --json
```

Capture options:

- `--duration-ms`: collection window length after capture starts.
- `--start-trigger first-byte`: wait for the first byte before starting the
  duration clock. This is the default for capture mode.
- `--start-trigger immediate`: start the duration clock when the command starts.
- `--initial-timeout-ms`: maximum wait for the first byte in first-byte mode.
  If omitted, the command uses `--timeout-ms`.
- `--idle-timeout-ms`: stop after this many quiet milliseconds once capture has
  started.
- `--max-bytes`: hard cap on the combined response bytes.

`write --read` accepts the same capture options after the write:

```bash
serial-mcp-server write --port <port> --baud 115200 --data RUN --read \
  --duration-ms 5000 --initial-timeout-ms 30000 --json
```

Macro automation commands:

```bash
serial-mcp-server macro validate --file examples/macros/ping.json --json
serial-mcp-server macro list --file examples/macros/ping.json --json
serial-mcp-server macro plan --file examples/macros/ping.json --macro ping --json
serial-mcp-server macro run --file examples/macros/ping.json --macro ping --dry-run --json
serial-mcp-server macro run --file examples/macros/ping.json --macro ping --simulate-read PONG --json
serial-mcp-server macro run --file examples/macros/ping.json --macro ping --port <port> --baud 115200 --json
```

Macro packs are JSON files with `schema_version` set to `0.3`. v0.3 supports `send`, `delay`, and `expect` steps inside macros, plus assemblies that call macros by name. `expect` supports `contains` and `equals`.

The macro DSL is intentionally restricted. It does not run shell commands, JavaScript, Python, file operations, loops, variables, if/else branches, Quick commands, or RTS/DTR macro steps.

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
| `read` | Read data from an open connection with timeout handling or a bounded capture window. |
| `close` | Close an open connection. |
| `set_control_lines` | Set RTS and/or DTR on an open connection. |
| `macro_load` | Validate and load an inline macro pack or pack file path into the server's in-memory registry. |
| `macro_list` | List loaded macro packs, macros, and assemblies. |
| `macro_unload` | Remove a loaded macro pack from the in-memory registry. |
| `macro_plan` | Expand a loaded, inline, or file-backed macro or assembly without opening hardware. |
| `macro_run` | Run a loaded macro or assembly against an existing connection or explicit simulation input. |
| `macro_run_inline` | Validate, plan, and run an inline macro pack without storing it in the registry. |

The MCP macro registry is runtime-only. Restarting the server clears loaded packs, and the server does not write a persistent macro library.

MCP `read` accepts these optional capture fields in addition to
`connection_id`, `timeout_ms`, `max_bytes`, and `encoding`:

```json
{
  "connection_id": "...",
  "duration_ms": 5000,
  "start_trigger": "first_byte",
  "initial_timeout_ms": 30000,
  "idle_timeout_ms": 1500,
  "max_bytes": 8192,
  "encoding": "utf8"
}
```

When `duration_ms` is absent, MCP `read` keeps the existing single-read
behavior. When `duration_ms` is present, the tool returns structured JSON text
with `completion_reason`, `waited_ms`, `elapsed_ms`, and `chunks`.

## Agent Skill

The repository includes a Claude Code and Codex compatible skill at:

```text
skills/serial-debug/
```

The skill is CLI-first and documents MCP as an optional configured path. It is intended for agents that need to list ports, run serial smoke tests, run macro automation, control RTS/DTR, or troubleshoot UART/USB-serial devices.

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
- Validation claims should be based on command output from the connected device.

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
cargo run --locked -- macro validate --file examples/macros/ping.json --json
cargo run --locked -- macro run --file examples/macros/ping.json --macro ping --simulate-read PONG --json
```

## License

This project is licensed under the MIT License. See [LICENSE](LICENSE).
