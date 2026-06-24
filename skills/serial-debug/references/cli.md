# CLI Reference

Use the installed `serial-mcp-server` binary. Prefer `--json` for agent, CI, and scripted workflows.

## Commands

```bash
serial-mcp-server serve
serial-mcp-server list-ports --json
serial-mcp-server probe --port <port> --baud 115200 --json
serial-mcp-server write --port <port> --baud 115200 --data H --read --timeout-ms 1000 --json
serial-mcp-server read --port <port> --baud 115200 --timeout-ms 1000 --json
serial-mcp-server set-control-lines --port <port> --rts high --dtr low --json
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

## Minimal Evidence Commands

```bash
serial-mcp-server --help
serial-mcp-server list-ports --json | jq .
serial-mcp-server write --help
serial-mcp-server set-control-lines --help
```
