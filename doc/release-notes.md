# Release Notes

## serial-mcp-server capture window draft

Status: local task branch, not released.

### Scope

This change adds a bounded capture window to serial reads while preserving the
existing single-read behavior when `duration_ms` is absent.

### User-Facing Changes

- `serial-mcp-server read` accepts `--duration-ms`,
  `--start-trigger immediate|first-byte`, `--initial-timeout-ms`, and
  `--idle-timeout-ms`.
- `serial-mcp-server write --read` accepts the same capture options.
- MCP `read` accepts `duration_ms`, `start_trigger`, `initial_timeout_ms`, and
  `idle_timeout_ms`.
- Capture responses include `completion_reason`, `waited_ms`, `elapsed_ms`, and
  `chunks`.

### Compatibility

- Existing reads without `duration_ms` continue to perform one serial read.
- `timeout_ms` remains a per-read wait timeout, not a total capture duration.

### AI / Agent Usage Disclosure

This is explicitly an AI/agent-facing feature. It is safe to describe it as a
bounded serial capture API for agents that need to choose how long to collect
data in one CLI or MCP call.

The precise behavior to disclose is:

- `duration_ms` controls how long data is collected after capture starts.
- The default `first_byte` trigger waits for output before starting the capture
  clock.
- `initial_timeout_ms` controls how long the caller is willing to wait for that
  first byte.
- `idle_timeout_ms` and `max_bytes` keep capture bounded.
- The response reports why collection stopped through `completion_reason`.

This is a request/response capture API. The implementation returns one combined
response after the bounded capture window completes.

### Validation

- `cargo fmt --check`: PASS
- `cargo clippy --locked --all-targets --all-features -- -D warnings`: PASS
- `cargo test --locked`: PASS
- `cargo doc --locked --all-features --no-deps`: PASS
- Read-only hardware validation on `/dev/cu.usbmodem101`
  (`Espressif USB JTAG/serial debug unit`) at 115200 baud: PASS

Hardware validation covered real read-only streaming, including one capture that
stopped by `max_bytes` and one capture that stopped by `duration_elapsed`. The
validated hardware scope for this capture-window change is read-only serial
capture.

## serial-mcp-server 0.2.0

Status: released.

### Scope

This release keeps the MCP stdio server and adds a scriptable CLI plus an agent skill.

### User-Facing Changes

- `serial-mcp-server serve` starts the MCP stdio server explicitly.
- `serial-mcp-server list-ports --json` lists available serial ports without MCP.
- `serial-mcp-server probe --port <port> --baud 115200 --json` opens and closes a port to check reachability.
- `serial-mcp-server write --port <port> --baud 115200 --data H --read --timeout-ms 1000 --json` writes data and optionally reads a response.
- `serial-mcp-server read --port <port> --baud 115200 --timeout-ms 1000 --json` reads data from a one-shot connection.
- `serial-mcp-server set-control-lines --port <port> --rts high --dtr low --json` requests RTS and DTR levels.
- `skills/serial-debug` provides a CLI-first workflow for Codex and Claude Code.

### Compatibility

- Existing no-subcommand MCP startup remains available for compatibility.
- New MCP client configurations should use `serve`.
- Rust MSRV is now documented and enforced as 1.74.

### Validation

```bash
cargo fmt --all -- --check
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo test --locked --all-targets --all-features
cargo doc --locked --all-features --no-deps
cargo install --path . --locked
cargo run --locked -- --version
cargo run --locked -- --help
cargo run --locked -- list-ports --json
uv run --with pyyaml python /Users/adan/.codex/skills/.system/skill-creator/scripts/quick_validate.py skills/serial-debug
```

Hardware validation is separate from software release gates. Validation claims
should reference command output captured against the connected device and should
name the tested board, command path, and serial settings.
