# STM32 Serial Communication Demo

![STM32G4 Development Board](img/stm32g4.jpg)

A comprehensive example demonstrating serial communication using the MCP serial server with real STM32 hardware.

## What This Demo Shows

This example demonstrates:
- **📡 Serial Communication**: Interactive command interface over USART
- **🔧 Hardware Control**: LED control and status monitoring via serial commands  
- **🧪 Complete MCP Testing**: Validates the 5 core MCP data-path tools with real hardware
- **📊 Interactive Interface**: Send commands to running firmware and get real-time responses

## Hardware Requirements

### Essential Hardware
- **STM32 Development Board**: STM32 board with USART1 capability (PA9/PA10)
- **USB-to-Serial Converter**: CH343 or similar (if board doesn't have built-in USB-UART)
- **USB Cables**: For connecting to PC and powering the board

### Connection
- Connect USART1 pins: PA9 (TX), PA10 (RX)
- Connect USB-to-Serial converter to PC
- Power the STM32 board
- LED connected to PB7 (built-in LED on most boards)

## Quick Demo

### 1. Build and Run the Firmware
```bash
cd examples/STM32_demo
cargo run --release
```

### 2. Use with MCP Serial Server
This demo is designed to work with the MCP serial server. The MCP server provides tools to:
- Discover available serial ports
- Open serial connections with proper configuration
- Send interactive commands to the firmware
- Receive and display responses
- Properly close connections
- Set RTS/DTR control lines when supported by the serial adapter

### 3. What You'll See
Once running, the demo provides:
- **Welcome message** with available commands
- **Interactive command interface** responding to user input
- **LED control** with status feedback
- **Counter functionality** with increment and reset
- **Blink patterns** for visual feedback

## Serial Configuration

- **Baud Rate**: 115200
- **Data Bits**: 8
- **Parity**: None
- **Stop Bits**: 1
- **Flow Control**: None

## Interactive Commands

The firmware implements an interactive command interface:

| Command | Function | Response |
|---------|----------|----------|
| `H` or `h` | Display help | Complete command list |
| `L` or `l` | Toggle LED state | "LED: ON" or "LED: OFF" |
| `C` or `c` | Show counter (increments) | "Counter: X" |
| `R` or `r` | Reset counter to 0 | "Counter reset to 0" |
| `B` or `b` | Blink LED 3 times | "Blinking LED 3 times..." |
| Other keys | Echo character | Character echoed back |

## MCP Tools Testing

This demo serves as a test platform for the 5 core MCP serial data tools:

- **Port Discovery** (1 tool): `list_ports` - Find available serial ports
- **Connection Management** (2 tools): `open`/`close` - Manage serial connections  
- **Communication** (2 tools): `write`/`read` - Send commands and receive responses
- **Control Lines** (1 tool): `set_control_lines` - Available in the server for adapters/devices that expose RTS/DTR

**✅ Core data-path tools tested successfully with 100% success rate**

## Technical Features

### Serial Implementation
- **Non-blocking USART**: Embassy-based async serial communication
- **Command parsing**: Single character and multi-byte command support
- **Real-time responses**: Immediate feedback for all commands
- **State management**: LED state and counter persistence

### Performance Characteristics  
- **Connection**: Reliable 115200 baud communication
- **Command Response**: <50ms latency for most commands
- **Throughput**: Sufficient for interactive debugging and control
- **Stability**: Tested for extended operation periods

## Documentation

Detailed testing documentation available in `docs/`:

- [Serial MCP Testing Results](docs/serial-mcp-testing-documentation.md) - Complete testing workflow and results

## Getting Started

### Prerequisites
1. **Hardware Setup**: Connect your STM32 board and USB-to-serial converter
2. **MCP Server**: Run the serial MCP server  
3. **Build and Run Firmware**: Use `cargo run --release` to compile and run

### Demo Features
- **Interactive Commands**: Real-time command processing
- **Hardware Control**: LED toggle and blink patterns
- **Counter System**: Increment and reset functionality
- **Help System**: Built-in command documentation

This is a demonstration example showing the capabilities of serial communication with real embedded hardware using the MCP serial server.

---

**Status: ✅ Hardware Validated - Real STM32 Testing on COM19**

Perfect for demonstrating professional embedded development workflows with interactive serial communication.
