# Progress

Current task: M1 serial capture window
Status: Local implementation complete; waiting for user review before PR
Branch: `task/serial-capture-window`

## Current Work

- Implemented capture windows for CLI `read`, CLI `write --read`, and MCP
  `read`.
- Updated README, README_ZH, theory docs, skill references, changelog, and
  release notes.
- Verification passed locally.
- Read-only hardware validation passed on `/dev/cu.usbmodem101`
  (`Espressif USB JTAG/serial debug unit`) at 115200 baud.
- Implementation commit: `fc86023`.
- User explicitly requested full local implementation and tests, with no PR
  until later confirmation.

## Log

- 2026-07-07T04:07:53Z: Started M1 on `task/serial-capture-window`.
- 2026-07-07T04:16:02Z: Passed `cargo fmt --check`,
  `cargo clippy --locked --all-targets --all-features -- -D warnings`,
  `cargo test --locked`, `cargo doc --locked --all-features --no-deps`,
  `cargo run --locked -- read --help`, and `cargo run --locked -- write --help`.
- 2026-07-07T04:16:55Z: Committed implementation as `fc86023`; no PR opened.
- 2026-07-07T05:55:46Z: Ran real hardware validation on
  `/dev/cu.usbmodem101` with `start-trigger=first-byte`: one capture stopped at
  `max_bytes` after 4096 bytes, and one capture stopped at `duration_elapsed`
  after 7656 bytes over 1202 ms. No data was written and RTS/DTR were not
  changed.
