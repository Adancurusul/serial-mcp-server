# Repository Agent Instructions

This repository is managed with Aragorn workflow state.

## Startup

Before workflow actions, read:

1. `doc/state.json`
2. `doc/progress.md`
3. `doc/project-info.md`
4. the last three entries in `doc/dev.md`
5. `doc/flow-config.json`
6. `doc/preferences.md`
7. `shared/custom/*.md` if present

If state is missing or inconsistent, route to project takeover or repair before
mutating source files.

## Workflow

- Canonical command entry: `/aragorn:flow`.
- Treat bare `/flow`, `/dev`, `/testing`, `/checkpoint`, and `/task-done` as
  documentation shorthands unless local aliases are installed.
- Use isolated git worktrees for source changes. Do not implement on `main`
  directly.
- Code changes require synchronized docs: roadmap, progress, dev log, and state.
- Testing and checkpoint phases require real CLI evidence.

## Project Rules

- Preserve MCP server behavior while adding CLI and skill surfaces.
- CLI/API surfaces come before GUI or client-specific workflow polish.
- Keep generated or volatile Aragorn runtime state out of commits.
- Documentation should use a direct engineering style and no emoji.
- Quality gates for release work:
  - `cargo fmt --all -- --check`
  - `cargo clippy --locked --all-targets --all-features -- -D warnings`
  - `cargo test --locked --all-targets --all-features`
  - `cargo doc --locked --all-features --no-deps`
