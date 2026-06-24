# M0 Quality Audit and Workflow Takeover

## Goal

Create durable Aragorn workflow state and perform a full quality audit before
source changes.

## Consumes / Produces

| Consumes | Produces |
| --- | --- |
| Existing Rust crate, README files, PR merge commits, baseline Cargo gate output | Aragorn docs, baseline evidence, audit backlog, M1 entry criteria |

## Approach

1. Reconstruct project facts from `Cargo.toml`, README files, source layout, and command output.
2. Record baseline quality gate results without changing source behavior.
3. Run a checkpoint-swarm style audit focused on Rust quality, CLI readiness, MCP behavior, docs truthfulness, and release risk.
4. Convert findings into M1-M5 tasks rather than repairing them inside M0.

## Module Boundaries

- Allowed to mutate: `doc/**`, `AGENTS.md`, `CLAUDE.md`, `.gitignore`.
- Do not mutate Rust source during M0 audit.
- Do not stage volatile `doc/workflow/context-artifacts/` files.

## Event Flow

```text
operator request
  -> Aragorn project takeover
  -> baseline Cargo commands
  -> M0 audit/checkpoint-swarm
  -> risk-ranked backlog
  -> M1 quality gates
```

## End-to-End Scenario

Command evidence for M0 baseline:

```bash
cargo fmt --all -- --check
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo test --locked --all-targets --all-features
cargo doc --locked --all-features --no-deps
```

Expected current baseline:

- fmt fails before M1.
- clippy fails before M1.
- tests pass.
- docs build with one warning before M1.

## Verification Checklist

| Item | Check |
| --- | --- |
| Project facts recorded | `rg -n "serial-mcp-server|MCP tools|set_control_lines" doc/project-info.md doc/requirements.md` |
| Baseline gates recorded | `rg -n "cargo fmt|cargo clippy|cargo test|cargo doc" doc/progress.md doc/dev.md` |
| Future CLI is not claimed as current | `rg -n "no user-facing serial operation subcommands yet" doc/project-info.md` |
| M1-M5 roadmap exists | `rg -n "M1: Engineering Quality Gates|M5: Release Readiness" doc/roadmap.md` |
| Runtime private state ignored | `rg -n "doc/workflow/context-artifacts|doc/.hook-state.json" .gitignore` |

## System Smoke Test

```bash
git diff --check
cargo test --locked --all-targets --all-features
```

M0 smoke does not require fmt or clippy to pass because those failures are the
documented input to M1.
