# Dev Log

## 2026-07-07T04:07:53Z

Task: M1 serial capture window
Branch: `task/serial-capture-window`

Plan:
- Implement backward-compatible blocking capture windows for CLI and MCP reads.
- Preserve existing single-read behavior when `duration_ms` is absent.
- Add docs, tests, and local commits.
- Do not open a PR until user confirmation.

Discovery:
- Existing `SerialConnection::read` performs one `stream.read` with timeout.
- CLI `read` and MCP `read` currently call the single-read primitive directly.
- `cargo test --locked` passed on the base checkout before implementation.

## 2026-07-07T04:16:02Z

Implemented:
- Added `src/serial/capture.rs` with capture configuration, completion reasons,
  chunk metadata, collector loop, and unit tests.
- Added `SerialConnection::capture`, which holds the serial stream mutex for
  the full capture window.
- Added CLI capture parameters to `read` and `write --read`.
- Added MCP `read` capture parameters and structured JSON text response for
  capture mode.
- Updated English/Chinese docs, theory docs, skill references, changelog, and
  release notes.

Verification:
- `cargo fmt --check` passed.
- `cargo clippy --locked --all-targets --all-features -- -D warnings` passed.
- `cargo test --locked` passed: 43 lib tests, 9 main tests, 9 macro automation
  tests, and doc-tests passed.
- `cargo doc --locked --all-features --no-deps` passed.
- `cargo run --locked -- read --help` and `cargo run --locked -- write --help`
  passed and showed capture options.

Notes:
- No hardware round-trip was run; this change is verified through unit/API/help
  gates only.
- No PR has been opened.
