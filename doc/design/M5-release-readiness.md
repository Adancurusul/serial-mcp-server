# M5 Release Readiness

## Goal

Prepare a new release after quality gates, CLI, skill, and documentation are
complete.

## Consumes / Produces

| Consumes | Produces |
| --- | --- |
| M1 clean gates, M2 CLI, M3 skill, M4 docs | Version bump, changelog, release notes, final release candidate evidence |

## Approach

- Decide version number after M2/M3 scope is complete. If CLI and skill ship
  together, `v0.2.0` is the likely next version.
- Update `Cargo.toml`, `Cargo.lock` if needed, README badges, changelog, and release notes together.
- Run all quality gates on the release candidate commit.
- Verify packaged help output and minimal CLI smoke.
- Keep hardware validation claims scoped to actual device evidence.

## Integration with Previous Milestones

M5 is the final integration milestone. It cannot start until M1-M4 are complete
or explicitly deferred with release notes.

## End-to-End Scenario

```bash
cargo fmt --all -- --check
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo test --locked --all-targets --all-features
cargo doc --locked --all-features --no-deps
cargo run --locked -- --version
cargo run --locked -- --help
```

If the CLI is implemented:

```bash
cargo run --locked -- list-ports --json
```

## Verification Checklist

| Item | Check |
| --- | --- |
| Version updated | `rg -n '^version = ' Cargo.toml` |
| README version status updated | `rg -n "v0\\.2\\.0|0\\.2\\.0|release" README.md README_ZH.md` |
| Changelog or release notes exist | `test -f CHANGELOG.md || test -f doc/release-notes.md` |
| Quality gates pass | M1 four-command gate sequence |
| CLI help works | `cargo run --locked -- --help` |
| Release claims scoped | `rg -n "tested|validated|hardware" README.md doc/release-notes.md` |

## System Smoke Test

```bash
cargo fmt --all -- --check && \
cargo clippy --locked --all-targets --all-features -- -D warnings && \
cargo test --locked --all-targets --all-features && \
cargo doc --locked --all-features --no-deps
```
