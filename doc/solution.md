# Solution

Implement capture windows as a layer above the existing single serial read.

## Design

- Keep `SerialConnection::read` as the single-read primitive.
- Add a capture collector that loops over single reads with:
  - total capture duration,
  - optional first-byte start trigger,
  - initial start timeout,
  - post-start idle timeout,
  - total byte cap,
  - per-chunk metadata.
- Reuse the collector from CLI `read`, CLI `write --read`, and MCP `read`.
- Preserve existing output fields while adding optional metadata fields.

## Non-Goals

- No background capture job API in this task.
- No MCP streaming/progress events in this task.
- No `serial-copilot` changes in this task.
- No unbounded capture duration.

