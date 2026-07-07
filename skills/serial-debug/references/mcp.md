# MCP Reference

Use MCP when the user asks for MCP or already has `serial-mcp-server` configured as an MCP stdio server. The CLI remains the fallback because it does not require client configuration.

## Server Command

```bash
serial-mcp-server serve
```

No-subcommand startup is kept compatible with existing stdio server behavior, but new documentation should prefer the explicit `serve` command.

## MCP Tools

- `list_ports`: list available serial ports.
- `open`: open a serial connection.
- `write`: write encoded data to an open connection.
- `read`: read data from an open connection with timeout handling or a bounded capture window.
- `close`: close an open connection.
- `set_control_lines`: set RTS and/or DTR on an open connection.
- `macro_load`: validate and load an inline macro pack or pack file path into the runtime registry.
- `macro_list`: list loaded macro packs, macros, and assemblies.
- `macro_unload`: remove a loaded macro pack from the runtime registry.
- `macro_plan`: expand a loaded, inline, or file-backed macro or assembly without opening hardware.
- `macro_run`: run a loaded macro or assembly with an existing connection or explicit simulation input.
- `macro_run_inline`: validate, plan, and run an inline macro pack without storing it in the registry.

The macro registry is in-memory only. Restarting the MCP server clears loaded macro packs.

## Capture Windows

MCP `read` keeps single-read behavior when `duration_ms` is absent. Add
`duration_ms` to collect one bounded window:

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

`start_trigger` can be `first_byte` or `immediate`. Capture responses include
`completion_reason`, `waited_ms`, `elapsed_ms`, and `chunks`.

## Agent Behavior

- Use MCP tool results as evidence only when the tool actually ran.
- Close connections after smoke tests.
- If an MCP client is not configured, switch to the CLI path and state that the CLI path was used.
- Do not mix MCP connection ids with CLI one-shot commands; CLI commands open and close their own connections.
- For macro workflows, call `macro_plan` before `macro_run` when the user asks for reviewable automation steps.
