# Requirements

## Scope

The project provides serial communication capabilities to AI clients through MCP
and will add a scriptable CLI and cross-agent skill without removing the MCP
server.

## Feature: Existing MCP Serial Operations

### Scenario: MCP client discovers serial ports

Given the `serial-mcp-server` binary is configured as an MCP stdio server
When an MCP client calls `list_ports`
Then the server returns the available serial port list or a structured serial error
And the server process remains available for additional tool calls

### Scenario: MCP client performs a serial data round trip

Given a reachable serial device is connected
And an MCP client has opened the port with `open`
When the client calls `write` with supported data encoding
And the client calls `read` with a timeout
Then the server returns the write count and decoded read payload
And the client can close the connection with `close`

### Scenario: MCP client controls RTS and DTR

Given a serial connection is open
When an MCP client calls `set_control_lines` with at least one of `rts` or `dtr`
Then the requested modem control line levels are applied
And the response records the requested state

## Feature: Quality Gates

### Scenario: Repository formatting is deterministic

Given the source tree is checked out from the release branch
When CI runs `cargo fmt --all -- --check`
Then the command exits successfully without source diffs

### Scenario: Clippy warnings are treated as release blockers

Given the source tree is checked out from the release branch
When CI runs `cargo clippy --locked --all-targets --all-features -- -D warnings`
Then the command exits successfully with no clippy warnings

### Scenario: Tests and docs are release evidence

Given the source tree is checked out from the release branch
When CI runs `cargo test --locked --all-targets --all-features`
And CI runs `cargo doc --locked --all-features --no-deps`
Then both commands exit successfully
And rustdoc does not emit warnings

## Feature: One-Shot CLI

### Scenario: User lists ports without MCP

Given the release binary is installed
When a user runs `serial-mcp-server list-ports --json`
Then stdout contains machine-readable port data
And stderr contains only diagnostics if needed
And the exit code is nonzero on failure

### Scenario: User probes a device without MCP

Given a reachable serial device is connected
When a user runs `serial-mcp-server probe --port <port> --baud 115200 --json`
Then the command opens the port, performs the configured probe, closes the port, and returns structured results

### Scenario: User writes and reads in one command

Given a reachable serial device is connected
When a user runs `serial-mcp-server write --port <port> --baud 115200 --data H --read --timeout-ms 1000 --json`
Then stdout contains write and read results
And all diagnostics are written to stderr

### Scenario: User starts MCP server explicitly

Given the release binary is installed
When a user runs `serial-mcp-server serve`
Then the process starts the existing MCP stdio server behavior

## Feature: Cross-Agent Serial Skill

### Scenario: Agent uses the skill through CLI

Given the serial debugging skill is installed in Claude Code or Codex
And the `serial-mcp-server` CLI is available
When a user asks the agent to list ports or run a serial smoke test
Then the skill instructs the agent to use the CLI first
And the agent returns concise evidence from command output

### Scenario: Agent uses MCP when configured

Given the serial debugging skill is installed
And an MCP client configuration for `serial-mcp-server` exists
When the user asks for an MCP-based serial operation
Then the skill may use MCP tools directly
And the skill keeps the CLI path documented as the fallback

## Feature: Documentation and Release

### Scenario: Documentation describes real surfaces

Given README and examples are refreshed
When a user reads the setup and usage docs
Then the docs distinguish implemented MCP tools, implemented CLI commands, and planned work
And the docs avoid emoji and broad production claims without evidence

### Scenario: A new version is releasable

Given quality gates, CLI, skill, and docs are complete
When the release checklist runs
Then the version metadata, changelog, release notes, and locked Cargo gates are consistent
And a release candidate can be tagged from `main`
