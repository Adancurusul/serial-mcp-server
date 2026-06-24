# M3 Cross-Agent Serial Skill

## Goal

Create a Claude Code and Codex compatible skill that lets agents operate serial
hardware through the CLI first, with MCP as an optional configured path.

## Consumes / Produces

| Consumes | Produces |
| --- | --- |
| M2 CLI commands, existing MCP setup docs, hardware safety notes | `serial-debug` skill folder with concise `SKILL.md` and references |

## Approach

- Keep `SKILL.md` short and action-oriented.
- Put detailed command examples in references loaded on demand.
- Prefer the CLI because it works without MCP client installation.
- Include MCP instructions for environments where the server is already configured.
- Do not bundle unrelated README or installation guides inside the skill folder.

## Planned Skill Layout

```text
skills/serial-debug/
  SKILL.md
  agents/openai.yaml
  references/
    cli.md
    mcp.md
    hardware-safety.md
    troubleshooting.md
  scripts/
    smoke-test.sh
```

`scripts/smoke-test.sh` is optional and should exist only if repeated command
sequencing proves fragile enough to require a deterministic script.

## Skill Trigger Contract

The skill should trigger for requests involving:

- serial port discovery
- UART or USB-serial smoke tests
- STM32, Arduino, ESP32, or similar board serial debugging
- RTS/DTR control line operations
- using `serial-mcp-server` from Claude Code or Codex

## Module Boundaries

- Skill assets may live under `skills/serial-debug/**` or another agreed release location.
- CLI reference must match the M2 implemented command surface.
- MCP reference must match the existing six MCP tools.
- Hardware safety reference must avoid unsupported claims and warn about voltage/line-level risks.

## Integration with Previous Milestones

M3 consumes the M2 CLI surface. It must not document commands that M2 has not
implemented. M3 also feeds M4 documentation by providing concise, tested usage
patterns.

## End-to-End Scenario

```bash
serial-mcp-server list-ports --json
serial-mcp-server probe --port <port> --baud 115200 --json
```

An agent using the skill should report command evidence and avoid pretending a
hardware smoke passed when no device was available.

## Verification Checklist

| Item | Check |
| --- | --- |
| Skill exists | `test -f skills/serial-debug/SKILL.md` |
| Frontmatter exists | `rg -n "^name:|^description:" skills/serial-debug/SKILL.md` |
| CLI reference exists | `test -f skills/serial-debug/references/cli.md` |
| MCP reference lists six tools | `rg -n "list_ports|set_control_lines" skills/serial-debug/references/mcp.md` |
| No extra skill docs | `find skills/serial-debug -maxdepth 1 -type f | rg 'README|CHANGELOG|INSTALL'` should find nothing |
| Skill validates | `scripts/quick_validate.py skills/serial-debug` if using the Codex skill template scripts |

## System Smoke Test

```bash
cargo test --locked --all-targets --all-features
serial-mcp-server list-ports --json
```

Skill validation should run after the skill folder exists.
