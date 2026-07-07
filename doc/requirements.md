# Requirements

## Feature: Serial Capture Window

Existing serial reads wait for one read operation and return as soon as data is
available. AI workflows need a single CLI or MCP call that can wait for device
output to start and then collect a bounded window of serial data.

### Scenario: Existing single read remains compatible

Given a caller does not pass `duration_ms`
When the caller invokes CLI `read` or MCP `read`
Then the command performs one serial read with the existing timeout behavior
And the existing response fields remain available

### Scenario: Immediate capture records a fixed window

Given a caller passes `duration_ms`
And `start_trigger` is `immediate`
When the caller invokes `read`
Then the capture window starts at invocation time
And serial chunks are collected until duration, idle timeout, or max bytes ends
the capture

### Scenario: First-byte capture waits before starting the window

Given a caller passes `duration_ms`
And `start_trigger` is `first_byte`
And `initial_timeout_ms` is greater than zero
When no data arrives before `initial_timeout_ms`
Then the response reports `completion_reason` as `initial_timeout`

When data arrives before `initial_timeout_ms`
Then that first data is included
And the `duration_ms` capture window starts from that first data

### Scenario: Capture reports why it stopped

Given a capture window is active
When the capture reaches duration, idle timeout, or max bytes
Then the response includes a machine-readable `completion_reason`
And includes chunk metadata with byte offsets and elapsed timing

### Scenario: Write-read supports capture windows

Given a caller invokes `write --read`
And passes capture window arguments
Then the write happens once
And the following read uses the same capture semantics as `read`

