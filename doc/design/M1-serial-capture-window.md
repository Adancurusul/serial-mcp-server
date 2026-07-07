# M1 Serial Capture Window Design

## Objective

Add a bounded, blocking capture mode to serial reads so an AI client can request
"wait for output, then collect N milliseconds of data" through the CLI or MCP.

## API

New read options:

- `duration_ms`: enables capture mode when present.
- `start_trigger`: `immediate` or `first_byte`.
- `initial_timeout_ms`: maximum wait for the first byte before the capture
  window starts.
- `idle_timeout_ms`: maximum quiet period after capture starts.
- `timeout_ms`: per-read wait timeout.
- `max_bytes`: existing total byte cap.

CLI spelling:

- `--duration-ms <ms>`
- `--start-trigger immediate|first-byte`
- `--initial-timeout-ms <ms>`
- `--idle-timeout-ms <ms>`

MCP spelling:

- `duration_ms`
- `start_trigger`: `"immediate"` or `"first_byte"`
- `initial_timeout_ms`
- `idle_timeout_ms`

## Response

Existing fields remain:

- `bytes_read`
- `data`
- `status`
- `timeout_ms`

Capture mode also reports:

- `elapsed_ms`: capture duration after the window starts.
- `waited_ms`: pre-window wait time.
- `completion_reason`: why capture stopped.
- `chunks`: byte offset, byte count, waited time, and capture elapsed time for
  each received chunk.

## Capture Completion Reasons

- `duration_elapsed`
- `initial_timeout`
- `idle_timeout`
- `max_bytes`

Single-read mode intentionally keeps the existing response shape. When
`duration_ms` is absent, `completion_reason` is not emitted.

## Implementation Notes

- Do not change the low-level single-read primitive.
- Use one collector shared by CLI and MCP to avoid semantic drift.
- Keep capture bounded by duration and max bytes.
- First-byte mode includes the first received bytes as chunk zero and starts
  capture timing from that moment.

## Verification Checklist

- Existing `read` without `duration_ms` still returns one read result.
- `duration_ms` with `immediate` continues after the first chunk.
- `duration_ms` with `first_byte` waits before starting elapsed capture time.
- `initial_timeout_ms` produces `initial_timeout`.
- `idle_timeout_ms` produces `idle_timeout` after capture starts.
- `max_bytes` produces `max_bytes`.
- CLI and MCP share response semantics.
- `cargo fmt --check` passes.
- `cargo test --locked` passes.
