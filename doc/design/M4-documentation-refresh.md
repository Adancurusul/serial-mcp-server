# M4 Documentation Refresh

## Goal

Refresh user-facing documentation so it accurately describes implemented MCP,
CLI, and skill surfaces in a direct engineering style.

## Consumes / Produces

| Consumes | Produces |
| --- | --- |
| M1 gates, M2 CLI, M3 skill | README, README_ZH, STM32 demo docs, safety notes, troubleshooting docs |

## Approach

- Remove emoji-heavy headings, bullets, and status labels.
- Replace broad production claims with evidence-backed status.
- Separate MCP setup, CLI usage, skill usage, hardware safety, and troubleshooting.
- Keep English and Chinese docs aligned on implemented features.
- Preserve useful STM32 demo instructions while updating command examples.

## Documentation Surfaces

- `README.md`
- `README_ZH.md`
- `CLAUDE.md`
- `examples/STM32_demo/README.md`
- `examples/STM32_demo/docs/serial-mcp-testing-documentation.md`
- skill references from M3

## Integration with Previous Milestones

M4 consumes the implemented CLI and skill from M2/M3. It must not describe
planned commands as available. M4 prepares the docs that M5 release notes will
point to.

## End-to-End Scenario

```bash
rg -n "[\\x{1F300}-\\x{1FAFF}]" README.md README_ZH.md examples/STM32_demo/README.md
cargo run --locked -- --help
cargo run --locked -- list-ports --help
```

The Unicode scan should return no emoji in refreshed docs. Command examples
must match real help output.

## Verification Checklist

| Item | Check |
| --- | --- |
| README has MCP setup | `rg -n "MCP|stdio|Claude" README.md` |
| README has CLI usage | `rg -n "list-ports|probe|--json" README.md` |
| README has skill usage | `rg -n "skill|Codex|Claude Code" README.md` |
| README_ZH matches surfaces | `rg -n "MCP|CLI|Codex|Claude Code" README_ZH.md` |
| Emoji removed | `rg -n "[\\x{1F300}-\\x{1FAFF}]" README.md README_ZH.md examples/STM32_demo/README.md` exits nonzero |
| Hardware safety documented | `rg -n "voltage|RTS|DTR|hardware safety|line level" README.md examples/STM32_demo/README.md` |

## System Smoke Test

```bash
cargo test --locked --all-targets --all-features
cargo run --locked -- --help
```
