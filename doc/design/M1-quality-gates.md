# M1 Engineering Quality Gates

## Goal

Make the repository pass the four release quality gates:

```bash
cargo fmt --all -- --check
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo test --locked --all-targets --all-features
cargo doc --locked --all-features --no-deps
```

## Consumes / Produces

| Consumes | Produces |
| --- | --- |
| M0 baseline failures and audit backlog | Clean formatting, clean clippy, clean tests, warning-free docs |

## Approach

- Apply rustfmt mechanically first.
- Resolve clippy warnings with small behavior-preserving changes.
- Fix rustdoc invalid HTML by using backticks for type names.
- Keep MSRV in view before accepting clippy suggestions that require newer APIs.
- Add or update tests only where a lint fix changes behavior boundaries.

## Known Baseline Findings

- `src/config.rs`: manual `Default` implementation can derive.
- `src/utils.rs`: `DataFormat::from_str` conflicts with `FromStr` expectations.
- `src/serial/mod.rs`: `connect` has too many arguments.
- `src/session/manager.rs`: nested `if` can collapse.
- `src/session/mod.rs`, `src/serial/tests.rs`, `src/tools/tests.rs`: module inception lints.
- `src/tools/serial_handler.rs`: manual multiple-of check; MSRV must be considered before using `.is_multiple_of`.
- `src/session/session.rs`: rustdoc warning for `Arc<Mutex>`.

## Module Boundaries

- Primary source: `src/config.rs`, `src/utils.rs`, `src/serial/**`, `src/session/**`, `src/tools/**`.
- Tests: existing unit tests under `src/**` and `tests/**`.
- Docs: update README or project docs only if quality-gate behavior or MSRV policy changes.

## Integration with Previous Milestones

M1 consumes M0's baseline evidence and must update M0's risk backlog with any
findings deferred beyond quality-gate cleanup.

## End-to-End Scenario

After implementation:

```bash
cargo fmt --all -- --check
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo test --locked --all-targets --all-features
cargo doc --locked --all-features --no-deps
```

All four commands must exit 0.

## Verification Checklist

| Item | Check |
| --- | --- |
| Formatting clean | `cargo fmt --all -- --check` |
| Clippy clean | `cargo clippy --locked --all-targets --all-features -- -D warnings` |
| Tests pass | `cargo test --locked --all-targets --all-features` |
| Docs warning-free | `cargo doc --locked --all-features --no-deps` |
| MSRV decision recorded | `rg -n "MSRV|rust-version|Rust 1.70" doc README.md Cargo.toml` |

## System Smoke Test

```bash
cargo fmt --all -- --check && \
cargo clippy --locked --all-targets --all-features -- -D warnings && \
cargo test --locked --all-targets --all-features && \
cargo doc --locked --all-features --no-deps
```
