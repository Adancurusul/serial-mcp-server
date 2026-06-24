---
name: serial-debug
description: CLI-first serial port debugging and macro automation with serial-mcp-server for Codex and Claude Code. Use when working with UART or USB-serial devices, listing serial ports, probing STM32/Arduino/ESP32 boards, writing or reading serial data, running JSON serial macros, controlling RTS/DTR, or using serial-mcp-server through MCP tools.
---

# Serial Debug

## Operating Rule

Use the `serial-mcp-server` CLI first unless the user explicitly asks for MCP or an MCP client is already configured. Keep stdout data separate from diagnostics, cite exact commands, and do not claim hardware success unless a real command touched the device and returned evidence.

## Workflow

1. Confirm the binary is available:

```bash
serial-mcp-server --help
```

2. List available ports before choosing a device:

```bash
serial-mcp-server list-ports --json
```

3. For a device smoke test, probe the port first, then write/read only when the probe succeeds:

```bash
serial-mcp-server probe --port <port> --baud 115200 --json
serial-mcp-server write --port <port> --baud 115200 --data H --read --timeout-ms 1000 --json
```

4. If RTS or DTR is involved, state the requested line levels and use the control-line command:

```bash
serial-mcp-server set-control-lines --port <port> --rts high --dtr low --json
```

5. For repeatable serial procedures, prefer JSON macro packs and validate or plan them before real hardware execution:

```bash
serial-mcp-server macro validate --file <pack.json> --json
serial-mcp-server macro plan --file <pack.json> --macro <name> --json
serial-mcp-server macro run --file <pack.json> --macro <name> --simulate-read <response> --json
serial-mcp-server macro run --file <pack.json> --macro <name> --port <port> --baud 115200 --json
```

Macro packs are restricted JSON DSL files. v0.3 supports `send`, `delay`, `expect contains`, `expect equals`, and assemblies that call macros. It does not support shell commands, language execution, loops, variables, Quick commands, or RTS/DTR macro steps.

## References

- Read `references/cli.md` for exact command behavior, output expectations, and automation rules.
- Read `references/mcp.md` when the user asks for MCP setup or MCP tool usage.
- Read `references/hardware-safety.md` before changing baud, wiring, voltage levels, RTS, DTR, or reset/boot lines.
- Read `references/troubleshooting.md` when a command fails, times out, returns no ports, or cannot open the device.

## Reporting

Report the command, exit status, and the relevant JSON or error excerpt. If no hardware is connected, say that only discovery/help paths were verified and mark device round-trip validation as manual.
