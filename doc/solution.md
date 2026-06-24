# Solution

## Context

`serial-mcp-server` is currently a Rust MCP stdio server with six serial tools
and configuration-oriented CLI flags. The next product direction is to keep MCP
while adding a scriptable CLI and a Claude Code / Codex compatible skill.

## Options

### Option A: MCP-only hardening

Keep the binary focused on MCP and only fix quality gates and docs.

Tradeoffs:

- Lowest implementation risk.
- Does not satisfy direct CLI + skill usage without MCP client setup.
- Keeps agent workflows dependent on MCP configuration.

### Option B: CLI-first expansion while retaining MCP

Add explicit subcommands for `serve`, `list-ports`, `probe`, `write`, `read`,
`set-control-lines`, `generate-config`, and `validate-config`. Keep the current
MCP handler and reuse shared serial logic.

Tradeoffs:

- Satisfies users who do not want to install MCP.
- Gives skills and CI a deterministic command surface.
- Requires careful command design to avoid duplicating serial behavior.

### Option C: Separate CLI crate plus MCP crate split

Split the project into a workspace with library, CLI, MCP server, and skill
packaging crates.

Tradeoffs:

- Clean long-term boundaries.
- Higher migration cost before the current quality baseline is clean.
- Not needed until the CLI surface stabilizes.

## Recommendation

Use Option B for the next version. First make the repository pass quality gates,
then add a one-shot CLI that calls shared serial operations, then add the skill
as a thin workflow layer over that CLI. Defer workspace/crate splitting until
there is enough duplication or release pressure to justify it.

## C4 Context

```text
User or AI agent
  -> serial-mcp-server CLI
  -> shared serial logic
  -> OS serial APIs
  -> serial hardware

MCP client
  -> serial-mcp-server MCP stdio transport
  -> MCP tool handler
  -> shared serial logic
  -> OS serial APIs
  -> serial hardware
```

## Container Shape

```text
Binary: serial-mcp-server
  - CLI parser and subcommands
  - MCP stdio server mode
  - config loading and validation

Library: serial_mcp_server
  - serial connection manager
  - serial connection wrapper
  - data encoding and decoding
  - MCP tool argument and response types

Skill: serial-debug
  - SKILL.md concise router
  - references/cli.md for command usage
  - references/mcp.md for MCP client path
  - references/hardware-safety.md for serial hardware precautions
  - references/troubleshooting.md for common failures
```

## Key Decisions

- MCP remains a first-class supported mode.
- CLI subcommands are one-shot first; no long-running CLI session manager in the first CLI milestone.
- CLI stdout is data only; diagnostics and progress go to stderr.
- JSON output is required for skill and CI workflows.
- Hardware safety notes belong in docs and skill references before release.
- The release milestone should not start until the four Cargo quality gates are clean.

## Requirement Mapping

| Requirement area | Solution component |
| --- | --- |
| MCP tools | Existing MCP handler, preserved and regression tested |
| Quality gates | M1 formatting, clippy, test, and rustdoc cleanup |
| Direct CLI usage | M2 subcommands over shared serial logic |
| Cross-agent skill | M3 skill folder using CLI first, MCP optional |
| Documentation | M4 README, examples, safety and troubleshooting docs |
| Release | M5 version, changelog, release notes, final gates |
