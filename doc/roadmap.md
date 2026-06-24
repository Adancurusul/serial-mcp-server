# Roadmap

## M0: Quality Audit and Workflow Takeover

- [x] M0.0 Record Aragorn takeover docs and baseline quality evidence.
- [ ] M0.1 Run checkpoint-swarm style full code quality analysis with Rust skills.
- [ ] M0.2 Convert audit findings into risk-ranked backlog items.
- [ ] M0.3 Decide MSRV policy: enforce Rust 1.70+ or update the documented minimum.

## M1: Engineering Quality Gates

- [ ] M1.1 Make `cargo fmt --all -- --check` pass.
- [ ] M1.2 Make `cargo clippy --locked --all-targets --all-features -- -D warnings` pass without weakening meaningful lint coverage.
- [ ] M1.3 Make `cargo doc --locked --all-features --no-deps` pass without warnings.
- [ ] M1.4 Add CI or local script documentation for the four release gates.
- [ ] M1.5 Integration task: rerun all four gates together on a clean checkout.

## M2: CLI Interface While Retaining MCP

- [ ] M2.1 Add explicit `serve` mode for existing MCP stdio behavior.
- [ ] M2.2 Add one-shot `list-ports`, `probe`, `write`, `read`, and `set-control-lines` commands.
- [ ] M2.3 Add `--json` output for script and skill consumption.
- [ ] M2.4 Preserve existing config generation, validation, and show behavior through compatible commands or flags.
- [ ] M2.5 Integration task: verify MCP mode and CLI mode can both operate from the same binary.

## M3: Cross-Agent Serial Skill

- [ ] M3.1 Create a concise skill design compatible with Claude Code and Codex.
- [ ] M3.2 Implement `SKILL.md` with CLI-first routing and MCP optional routing.
- [ ] M3.3 Add references for CLI commands, MCP setup, hardware safety, and troubleshooting.
- [ ] M3.4 Add a smoke-test script only if repeated command sequencing needs deterministic reliability.
- [ ] M3.5 Integration task: validate the skill against the M2 CLI and existing MCP documentation.

## M4: Documentation Refresh

- [ ] M4.1 Rewrite README and README_ZH in direct engineering style.
- [ ] M4.2 Remove emoji-heavy formatting and status claims not backed by evidence.
- [ ] M4.3 Document MCP setup, CLI usage, skill usage, hardware safety, and troubleshooting.
- [ ] M4.4 Refresh STM32 demo docs to match current MCP and CLI surfaces.
- [ ] M4.5 Integration task: run docs commands and verify examples reference implemented commands only.

## M5: Release Readiness

- [ ] M5.1 Decide and apply version bump, likely v0.2.0 if CLI and skill ship together.
- [ ] M5.2 Update changelog and release notes.
- [ ] M5.3 Run the full quality gate matrix.
- [ ] M5.4 Verify packaged binary help, MCP startup, CLI smoke, and skill docs.
- [ ] M5.5 Integration task: create a release candidate from `main` with all gates passing.
