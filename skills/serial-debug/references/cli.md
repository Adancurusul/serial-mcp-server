# CLI Reference

Use the installed `serial-mcp-server` binary. Prefer `--json` for agent, CI, and scripted workflows.

## Commands

```bash
serial-mcp-server serve
serial-mcp-server list-ports --json
serial-mcp-server probe --port <port> --baud 115200 --json
serial-mcp-server write --port <port> --baud 115200 --data H --read --timeout-ms 1000 --json
serial-mcp-server read --port <port> --baud 115200 --timeout-ms 1000 --json
serial-mcp-server read --port <port> --baud 115200 --duration-ms 5000 --initial-timeout-ms 30000 --idle-timeout-ms 1500 --json
serial-mcp-server set-control-lines --port <port> --rts high --dtr low --json
serial-mcp-server macro validate --file <pack.json> --json
serial-mcp-server macro list --file <pack.json> --json
serial-mcp-server macro plan --file <pack.json> --macro <name> --json
serial-mcp-server macro run --file <pack.json> --macro <name> --dry-run --json
serial-mcp-server macro run --file <pack.json> --macro <name> --simulate-read <response> --json
serial-mcp-server macro run --file <pack.json> --macro <name> --port <port> --baud 115200 --json
serial-mcp-server generate-config
serial-mcp-server validate-config --config <path>
serial-mcp-server show-config --config <path>
```

## Output Rules

- Treat stdout as data. JSON stdout should parse with `jq`.
- Treat stderr as diagnostics. Cargo wrapper output is not binary output; when testing the installed binary, run `target/debug/serial-mcp-server` or the release binary directly.
- Nonzero exit means the operation failed. Report the failure with the stderr excerpt and do not infer hardware state.

## Data Formats

The `write` and `read` commands support:

- `--format utf8`
- `--format hex`
- `--format base64`

Use hex or base64 for binary-looking payloads. Use UTF-8 only when the protocol is text based.

## Capture Windows

`--timeout-ms` waits for one read operation. Use `--duration-ms` when you need
one bounded capture window that continues after the first bytes arrive.

```bash
serial-mcp-server read --port <port> --baud 115200 \
  --duration-ms 5000 \
  --start-trigger first-byte \
  --initial-timeout-ms 30000 \
  --idle-timeout-ms 1500 \
  --max-bytes 8192 \
  --json
```

Capture mode returns combined data plus `completion_reason`, `waited_ms`,
`elapsed_ms`, and `chunks`. `write --read` accepts the same capture options.

## Macro Packs

Use macro packs for repeatable serial procedures that an agent may need to validate, inspect, and run later.

Minimal shape:

```json
{
  "schema_version": "0.3",
  "name": "ping-pack",
  "macros": [
    {
      "name": "ping",
      "steps": [
        { "type": "send", "data": "PING\n", "encoding": "utf8" },
        { "type": "expect", "op": "contains", "data": "PONG", "encoding": "utf8" }
      ]
    }
  ]
}
```

Validate and plan before real hardware execution. Use `--simulate-read` when no device is attached.

## Minimal Evidence Commands

```bash
serial-mcp-server --help
serial-mcp-server list-ports --json | jq .
serial-mcp-server write --help
serial-mcp-server set-control-lines --help
serial-mcp-server macro validate --file examples/macros/ping.json --json
serial-mcp-server macro run --file examples/macros/ping.json --macro ping --simulate-read PONG --json
```
