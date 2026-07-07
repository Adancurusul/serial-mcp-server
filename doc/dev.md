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

## 2026-07-07T04:16:55Z

Committed:
- `fc86023` feat: add serial capture windows

Status:
- Local branch implementation is complete.
- Awaiting user review and explicit confirmation before opening a PR.

## 2026-07-07T05:55:46Z

Hardware validation:
- Detected `/dev/cu.usbmodem101` as `Espressif USB JTAG/serial debug unit`
  (`USB VID:303A PID:1001`).
- `target/debug/serial-mcp-server probe --port /dev/cu.usbmodem101 --baud 115200 --json`
  succeeded with `opened=true`.
- Capture test 1:
  `target/debug/serial-mcp-server read --port /dev/cu.usbmodem101 --baud 115200 --format hex --timeout-ms 250 --duration-ms 3000 --start-trigger first-byte --initial-timeout-ms 5000 --idle-timeout-ms 1000 --max-bytes 4096 --json`
  returned `bytes_read=4096`, `waited_ms=120`, `elapsed_ms=644`,
  `completion_reason=max_bytes`, and `chunks_len=86`.
- Capture test 2:
  `target/debug/serial-mcp-server read --port /dev/cu.usbmodem101 --baud 115200 --format hex --timeout-ms 250 --duration-ms 1200 --start-trigger first-byte --initial-timeout-ms 3000 --idle-timeout-ms 1000 --max-bytes 65536 --json`
  returned `bytes_read=7656`, `waited_ms=121`, `elapsed_ms=1202`,
  `completion_reason=duration_elapsed`, and `chunks_len=137`.
- The decoded stream prefix contained sensor-style lines such as
  `[T: 9964.026703125] AX: +0.01 AY: -0.02 AZ: -0.96 ...`.
- No write command was run. RTS/DTR were not changed.
