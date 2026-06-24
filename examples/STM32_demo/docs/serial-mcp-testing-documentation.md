# Serial MCP STM32 Test Notes

This document records the STM32 demo test shape for `serial-mcp-server`. Treat it as historical hardware evidence unless you re-run the commands on the current hardware setup.

## Test Environment

| Field | Value |
| --- | --- |
| Target firmware | `examples/STM32_demo` |
| Example hardware | STM32 board with CH343 USB-to-serial adapter |
| Historical port | `COM19` |
| Serial settings | 115200 baud, 8N1, no flow control |
| Surfaces | MCP tools and CLI one-shot commands |

## Firmware Behavior

The firmware accepts these commands:

- `H` or `h`: print help.
- `L` or `l`: toggle LED state.
- `C` or `c`: print and increment counter.
- `R` or `r`: reset counter.
- `B` or `b`: blink LED three times.
- Other input: echo the character.

## MCP Test Sequence

1. `list_ports`: confirm the target adapter is visible.
2. `open`: open the target port at `115200`.
3. `write`: send `H`, `L`, or `C`.
4. `read`: capture the firmware response.
5. `set_control_lines`: optional, only when RTS/DTR behavior is intentionally under test.
6. `close`: close the connection.

Expected responses include the help text, `LED: ON` or `LED: OFF`, and `Counter: <n>`.

## CLI Test Sequence

```bash
serial-mcp-server list-ports --json
serial-mcp-server probe --port <port> --baud 115200 --json
serial-mcp-server write --port <port> --baud 115200 --data H --read --timeout-ms 2000 --json
```

Use `jq` to verify JSON stdout:

```bash
serial-mcp-server list-ports --json | jq .
```

## Release Evidence Rule

Only claim the current hardware has been validated when the current run includes command output from the connected device. If no board is connected, record CLI help, port discovery, and software tests separately from manual hardware validation.
