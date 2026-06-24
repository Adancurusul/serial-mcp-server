# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Run Commands

```bash
# Build release binary
cargo build --release

# Run the server directly (uses stdio transport for MCP)
cargo run --release

# Run tests
cargo test

# Run a specific test
cargo test test_name

# Generate default configuration file
cargo run --release -- --generate-config

# Validate configuration
cargo run --release -- --validate-config

# Show current configuration
cargo run --release -- --show-config

# Run with custom config file
cargo run --release -- --config path/to/config.toml

# Run with debug logging
RUST_LOG=debug cargo run --release
```

## Architecture Overview

This is a Model Context Protocol (MCP) server for serial port communication, built with Rust using the `rmcp` SDK.

### Core Components

- **`src/main.rs`** - Entry point. Handles CLI argument parsing, logging initialization, and MCP server startup using stdio transport.

- **`src/tools/serial_handler.rs`** - MCP tool handler implementing `ServerHandler` trait. Defines the 6 MCP tools (`list_ports`, `open`, `close`, `write`, `read`, `set_control_lines`) using the `#[tool]` macro from rmcp.

- **`src/serial/mod.rs`** - `ConnectionManager` manages active serial connections using a thread-safe `HashMap<String, Arc<SerialConnection>>`. Handles connection lifecycle.

- **`src/serial/connection.rs`** - `SerialConnection` wraps the actual serial port with async read/write operations using tokio-serial.

- **`src/config.rs`** - Configuration system with CLI args (clap) and TOML file support. `Config` struct has nested configs: `ServerConfig`, `SerialConfig`, `SecurityConfig`, `LoggingConfig`.

### Data Flow

```
MCP Client (AI)
    → stdio transport
    → SerialHandler (tool methods)
    → ConnectionManager
    → SerialConnection
    → Physical serial device
```

### Key Patterns

- Tools are defined with `#[tool]` macro and registered via `#[tool_router]` on the handler impl
- Connection IDs are UUIDs returned by `open` and required for `write`/`read`/`close`/`set_control_lines`
- Data encoding supports utf8, hex, and base64 for read/write operations
- RTS/DTR modem control lines can be set per open serial connection with `set_control_lines`
- Async operations use tokio with `RwLock` for connection state

### STM32 Demo

Located at `examples/STM32_demo/` - embedded firmware example for testing serial communication with real hardware.
