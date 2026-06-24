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
- `read`: read data from an open connection with timeout handling.
- `close`: close an open connection.
- `set_control_lines`: set RTS and/or DTR on an open connection.

## Agent Behavior

- Use MCP tool results as evidence only when the tool actually ran.
- Close connections after smoke tests.
- If an MCP client is not configured, switch to the CLI path and state that the CLI path was used.
- Do not mix MCP connection ids with CLI one-shot commands; CLI commands open and close their own connections.
