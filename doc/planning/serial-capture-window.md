# Serial Capture Window Plan

## Problem

The existing `timeout_ms` means "wait for one serial read". It does not mean
"collect data for this many milliseconds". A device that emits once per second
causes a read with `timeout_ms = 5000` to return after the first emission.

## Goal

Add a backward-compatible capture mode that allows AI clients to request a
bounded collection window from CLI and MCP.

## First Version Scope

Included:

- CLI `read` capture windows.
- CLI `write --read` capture windows.
- MCP `read` capture windows.
- Immediate and first-byte start triggers.
- Initial timeout, idle timeout, max bytes, chunk metadata.
- Documentation and tests.

Excluded:

- Background capture jobs.
- MCP streaming/progress events.
- `serial-copilot` changes.
- Unbounded capture.

## Handoff Prompt

Continue in `/Users/adan/adan-ws/github-code/serial-mcp-server-task-serial-capture-window`.
The task is M1 serial capture window on branch `task/serial-capture-window`.
Implement and test backward-compatible CLI/MCP read capture windows:
`duration_ms`, `start_trigger`, `initial_timeout_ms`, `idle_timeout_ms`,
`max_bytes`, chunk metadata, waited/elapsed timing, and completion reason.
Do not open a PR. Run `cargo fmt --check` and `cargo test --locked`.

