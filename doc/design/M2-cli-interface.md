# M2 CLI Interface While Retaining MCP

## Goal

Add a direct, scriptable CLI while preserving the existing MCP stdio server.

## Consumes / Produces

| Consumes | Produces |
| --- | --- |
| M1 clean gates, existing MCP serial logic | CLI subcommands, JSON output, MCP-compatible `serve` mode |

## Approach

- Introduce subcommands with `clap::Subcommand`.
- Make `serve` the explicit MCP stdio mode while preserving compatibility for existing no-subcommand MCP startup if required.
- Reuse shared serial connection and data encoding logic.
- Start with one-shot operations. Do not add an interactive session daemon in M2.
- Keep stdout data-only and diagnostics on stderr.

## Command Schemas

Planned commands:

```text
serial-mcp-server serve
serial-mcp-server list-ports [--json]
serial-mcp-server probe --port <port> --baud <baud> [--json]
serial-mcp-server write --port <port> --baud <baud> --data <data> [--format utf8|hex|base64] [--read] [--timeout-ms <ms>] [--json]
serial-mcp-server read --port <port> --baud <baud> [--timeout-ms <ms>] [--json]
serial-mcp-server set-control-lines --port <port> [--rts high|low] [--dtr high|low] [--json]
serial-mcp-server generate-config
serial-mcp-server validate-config [--config <path>]
```

## Authority Matrix

| Surface | Authority |
| --- | --- |
| MCP tool semantics | Existing MCP handler and tests |
| CLI argument syntax | M2 design and clap help snapshots |
| JSON output | M2 tests and skill requirements |
| Hardware behavior | Manual or hardware-in-loop evidence only |

## Module Boundaries

- `src/config.rs`: extend CLI args and subcommands.
- `src/main.rs`: dispatch serve vs one-shot commands.
- `src/cli/**` or equivalent: command execution if added.
- `src/serial/**` and `src/tools/types.rs`: reuse shared logic; avoid divergent encoders.
- Tests under `src/**` or `tests/**` must cover command parsing and JSON shape.

## Integration with Previous Milestones

M2 must keep all M1 gates passing. M2 also sets up the stable command surface
consumed by M3 skill references.

## End-to-End Scenario

```bash
serial-mcp-server --help
serial-mcp-server serve --help
serial-mcp-server list-ports --json
```

For hardware smoke when a device is available:

```bash
serial-mcp-server write --port <port> --baud 115200 --data H --read --timeout-ms 1000 --json
```

## Verification Checklist

| Item | Check |
| --- | --- |
| Serve mode exists | `cargo run --locked -- serve --help` |
| Port listing exists | `cargo run --locked -- list-ports --help` |
| JSON flag exists | `cargo run --locked -- list-ports --help | rg -- '--json'` |
| Write/read command exists | `cargo run --locked -- write --help` |
| MCP behavior preserved | `cargo run --locked -- serve --help` and existing MCP tests |
| Quality gates still pass | M1 four-command gate sequence |

## System Smoke Test

```bash
cargo test --locked --all-targets --all-features
cargo run --locked -- --help
cargo run --locked -- serve --help
cargo run --locked -- list-ports --json
```
