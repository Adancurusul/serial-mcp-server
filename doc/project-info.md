# Project Info

## Shape

- Name: `serial-mcp-server`
- Type: Rust MCP stdio server for serial port communication.
- Package: `serial-mcp-server` v0.1.0, Rust 2021, MIT license.
- Repository: `https://github.com/Adancurusul/serial-mcp-server`
- Primary source:
  - `src/main.rs`: clap args, config loading, MCP stdio startup, shutdown cleanup.
  - `src/tools/serial_handler.rs`: MCP tool handler and tool registration.
  - `src/tools/types.rs`: tool argument and response types.
  - `src/serial/**`: serial connection, connection manager, port listing.
  - `src/session/**`: session abstractions currently present but not the primary MCP surface.
  - `src/config.rs`: CLI flags, TOML config, validation.
- Example hardware project: `examples/STM32_demo/`.

## Current Consumable Surface

- MCP server over stdio via `serial-mcp-server` with no subcommand.
- MCP tools:
  - `list_ports`
  - `open`
  - `write`
  - `read`
  - `close`
  - `set_control_lines`
- Configuration CLI flags:
  - `--generate-config`
  - `--validate-config`
  - `--show-config`
  - `--config <path>`
  - runtime defaults such as `--default-baud-rate`, `--default-timeout-ms`, and `--max-buffer-size`.
- Current binary has no user-facing serial operation subcommands yet.

## Current Milestone Intent

| Field | Value |
| --- | --- |
| Milestone | M0 quality audit and workflow takeover |
| Goal | Establish Aragorn-managed project state and produce an evidence-based quality backlog before source changes. |
| Hypothesis | The repo is usable as an MCP server today, but release-grade quality gates and a CLI/skill surface need explicit staged work. |
| Boundary | M0 records facts and audit evidence only; it does not change Rust behavior, public protocol, or release version. |

## Shaping Decisions

- Preserve MCP as the integration path for AI clients.
- Add a one-shot CLI for humans, scripts, CI, and agent skills.
- Build a Claude Code and Codex compatible skill around the CLI first, with MCP as an optional configured path.
- Treat release readiness as a separate milestone after quality gates, CLI, skill, and docs are complete.
- Keep documentation direct and engineering-oriented; future docs should remove emoji-heavy marketing language.

## Baseline Quality Snapshot

- `cargo test --locked --all-targets --all-features`: PASS, 37 library tests and 2 binary tests.
- `cargo doc --locked --all-features --no-deps`: PASS with one rustdoc warning for `Arc<Mutex>` not being backticked.
- `cargo fmt --all -- --check`: FAIL due to repository-wide rustfmt diffs.
- `cargo clippy --locked --all-targets --all-features -- -D warnings`: FAIL due to clippy warnings promoted to errors.

## Open Questions

- Decide whether README's Rust 1.70+ badge is a real MSRV contract or should be updated and enforced with `rust-version`.
- Decide whether the CLI should be a single binary with subcommands under `serial-mcp-server` or a second binary.
- Decide the first supported skill installation target: in-repo skill folder, user-global skill, or both.
