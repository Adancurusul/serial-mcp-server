# Preferences

## Tooling

- Package manager: Cargo with the checked-in `Cargo.lock`.
- Formatter: `cargo fmt --all -- --check`.
- Linter: `cargo clippy --locked --all-targets --all-features -- -D warnings`.
- Tests: `cargo test --locked --all-targets --all-features`.
- Docs: `cargo doc --locked --all-features --no-deps`.
- Search: prefer `rg` and `rg --files`.

## Workflow

- Use Aragorn as the workflow authority.
- Canonical command entry: `/aragorn:flow`.
- Use isolated git worktrees for source changes.
- Keep `main` as the base branch.
- Use real command output as completion evidence.
- Keep volatile Aragorn runtime state out of commits.

## Rust Policy

- Prefer small, behavior-preserving fixes for M1 quality gates.
- Do not use newer Rust APIs to satisfy clippy if they conflict with the documented MSRV.
- Resolve the MSRV question before release; either enforce Rust 1.70 compatibility or update documentation and `Cargo.toml`.
- CLI behavior should reserve stdout for data and JSON output; diagnostics and progress go to stderr.
- Nonzero exit codes are required for command failures.

## Documentation Policy

- Write internal Aragorn workflow docs in English.
- Keep user-facing docs direct and engineering-oriented.
- Do not use emoji in new or refreshed documentation.
- Do not claim hardware validation beyond recorded evidence.
- Keep implementation status distinct from planned milestones.

## Forbidden Paths and Data

- Do not commit secrets, `.env` files, or machine-local credentials.
- Do not stage `target/`.
- Do not stage `doc/.hook-state.json`.
- Do not stage `doc/workflow/context-artifacts/`.
- Do not stage `doc/workflow/workspaces/index.json`.

## Auxiliary Skills

- `aragorn`: workflow routing, state, phase gates, checkpoint and checkpoint-swarm planning.
- `rust-skills`: Rust routing, CLI domain constraints, rustfmt/clippy guidance.
- `skill-creator`: future Claude Code and Codex compatible serial debugging skill design.

## Concept Anchors

- Carmack-style status: report what changed, why, and what evidence exists.
- BurntSushi-style reviewability: each delivery should be a complete, self-contained unit.
- Unix-style interface design: one command surface should be scriptable, composable, and quiet by default.
