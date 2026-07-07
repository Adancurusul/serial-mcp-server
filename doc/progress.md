# Progress

Current task: M1 serial capture window
Status: Implementation and tests complete; local commit pending
Branch: `task/serial-capture-window`

## Current Work

- Implemented capture windows for CLI `read`, CLI `write --read`, and MCP
  `read`.
- Updated README, README_ZH, theory docs, skill references, changelog, and
  release notes.
- Verification passed locally.
- User explicitly requested full local implementation and tests, with no PR
  until later confirmation.

## Log

- 2026-07-07T04:07:53Z: Started M1 on `task/serial-capture-window`.
- 2026-07-07T04:16:02Z: Passed `cargo fmt --check`,
  `cargo clippy --locked --all-targets --all-features -- -D warnings`,
  `cargo test --locked`, `cargo doc --locked --all-features --no-deps`,
  `cargo run --locked -- read --help`, and `cargo run --locked -- write --help`.
