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

