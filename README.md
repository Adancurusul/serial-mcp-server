# Serial MCP Server

[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://rust-lang.org)
[![RMCP](https://img.shields.io/badge/RMCP-0.3.2-blue.svg)](https://github.com/modelcontextprotocol/rust-sdk)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)

A professional Model Context Protocol (MCP) server for serial port communication. Provides AI assistants with comprehensive serial communication capabilities for embedded systems, IoT devices, and hardware debugging with real hardware integration.

> 📖 **Language Versions**: [English](README.md) | [中文](README_ZH.md)

## ✨ Features

- 🚀 **Production Ready**: Real hardware integration with 5 comprehensive serial communication tools
- 🔌 **Cross-Platform Support**: Windows, Linux, macOS with automatic port detection
- 📡 **Complete Serial Control**: List ports, connect, send/receive data with full configuration
- 📝 **Multiple Data Formats**: UTF-8, Hex, Binary encoding support with timeout handling
- 🛠️ **Hardware Integration**: Tested with STM32, Arduino, ESP32 and other embedded systems
- 🤖 **AI Integration**: Perfect compatibility with Claude and other AI assistants
- 🧪 **Comprehensive Testing**: All 5 tools validated with real hardware 
- ⚡ **High Performance**: Built on Tokio async runtime with concurrent connection support

## 🏗️ Architecture

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   MCP Client    │◄──►│  Serial MCP      │◄──►│  Serial Device  │
│   (Claude/AI)   │    │  Server          │    │  Hardware       │
└─────────────────┘    └──────────────────┘    └─────────────────┘
                              │
                              ▼
                       ┌──────────────────┐
                       │  Target Device   │
                       │  (STM32/Arduino) │
                       └──────────────────┘
```

## 🚀 Quick Start

### Prerequisites

**Hardware Requirements:**
- **Serial Device**: STM32, Arduino, ESP32, or any UART-compatible device
- **Connection**: USB-to-Serial converter or built-in USB-UART
- **USB Cables**: For connecting device to PC

**Software Requirements:**
- Rust 1.70+ 
- Serial device drivers (automatically detected on most systems)

### Installation

```bash
# Clone and build from source
git clone https://github.com/adancurusul/serial-mcp-server.git
cd serial-mcp-server
cargo build --release
```

### Basic Usage

**Configure MCP Clients**

#### Claude Desktop Configuration Example

Add to Claude Desktop configuration file:

**Windows Example:**
```json
{
  "mcpServers": {
    "serial": {
      "command": "C:\\path\\to\\serial-mcp-server\\target\\release\\serial-mcp-server.exe",
      "args": [],
      "env": {
        "RUST_LOG": "info"
      }
    }
  }
}
```

**macOS/Linux Example:**
```json
{
  "mcpServers": {
    "serial": {
      "command": "/path/to/serial-mcp-server/target/release/serial-mcp-server",
      "args": [],
      "env": {
        "RUST_LOG": "info"
      }
    }
  }
}
```

Other examples for other tools like cursor, claude code etc. please refer to the corresponding tool documentation

## 🎯 Try the STM32 Demo

We provide a comprehensive **STM32 Serial Communication Demo** that showcases all capabilities:

```bash
# Navigate to the example
cd examples/STM32_demo

# Build and run the firmware  
cargo run --release

# Use with MCP server for complete serial communication experience
```

**What the demo shows:**
- ✅ **Interactive Serial Commands**: Send commands and get real-time responses
- ✅ **All 5 MCP Tools**: Complete validation with real STM32 hardware
- ✅ **Hardware Control**: LED toggle, counter system, blink patterns
- ✅ **Command Interface**: Help system with interactive command processing

[📖 View STM32 Demo Documentation →](examples/STM32_demo/README.md)

### Usage Examples with AI Assistants

#### List Available Serial Ports
```
Please list available serial ports on the system
```

#### Connect to Serial Device
```
Connect to COM19 with baud rate 115200 for my STM32 device
```

#### Send Interactive Commands
```
Send 'H' command to get help menu, then send 'L' to toggle the LED
```

#### Read Device Responses
```
Read the response from the serial device with 2 second timeout
```

#### Complete Communication Test
```
Please help me test all 5 MCP serial tools with my STM32 board on COM19. Start by listing ports, then connect, send some commands, read responses, and finally close the connection.
```

## 🛠️ Complete Tool Set (5 Tools)

All tools tested and validated with real STM32 hardware:

### 📡 Serial Communication (5 tools)
| Tool | Description | Status |
|------|-------------|----------|
| `list_ports` | Discover available serial ports on system | ✅ Production Ready |
| `open` | Open serial connection with configuration | ✅ Production Ready |
| `write` | Send data to connected serial device | ✅ Production Ready |
| `read` | Read data from serial device with timeout | ✅ Production Ready |
| `close` | Close serial connection cleanly | ✅ Production Ready |

**✅ 5/5 Tools - 100% Success Rate with Real Hardware**

## 🌍 Supported Hardware

### Serial Devices
- **STM32**: All STM32 series with UART capability  
- **Arduino**: Uno, Nano, ESP32, ESP8266
- **Embedded Systems**: Any device with UART/USB-Serial interface
- **Industrial**: Modbus, RS485 converters
- **IoT Devices**: Sensors, actuators with serial communication
- **Other**: Any device with UART/USB-Serial interface

### Serial Interfaces
- **USB-to-Serial**: CH340, CH343, FTDI, CP2102
- **Built-in USB**: STM32 with USB-CDC, Arduino Leonardo
- **Hardware UART**: Direct UART connections

### Platform Support

| Platform | Port Format | Examples |
|----------|-------------|----------|
| Windows | `COMx` | COM1, COM3, COM19 |
| Linux | `/dev/ttyXXX` | /dev/ttyUSB0, /dev/ttyACM0 |
| macOS | `/dev/tty.xxx` | /dev/tty.usbserial-1234 |

## 🏆 Production Status

### ✅ Fully Implemented and Tested

**Current Status: PRODUCTION READY**

- ✅ **Complete Serial Integration**: Real hardware communication with all 5 tools
- ✅ **Hardware Validation**: Tested with STM32 + CH343 USB-Serial on COM19
- ✅ **Interactive Communication**: Full bidirectional command/response system
- ✅ **Multi-Platform**: Windows, Linux, macOS support with automatic detection
- ✅ **Connection Management**: Robust connection handling with proper cleanup
- ✅ **AI Integration**: Perfect MCP protocol compatibility

## 📦 Technical Features

### Serial Implementation
- **Cross-Platform**: Automatic port detection and configuration
- **Multiple Encodings**: UTF-8, Hex, Binary data support
- **Timeout Handling**: Configurable read/write timeouts
- **Connection Pooling**: Multiple concurrent serial connections

### Performance Characteristics
- **Port Discovery**: Fast enumeration of available ports
- **Connection Speed**: Rapid connection establishment
- **Data Throughput**: Efficient data transfer with minimal latency
- **Session Stability**: Tested for extended operation periods

## 🙏 Acknowledgments

Thanks to the following open source projects:

- [serialport-rs](https://crates.io/crates/serialport) - Serial port communication library
- [rmcp](https://github.com/modelcontextprotocol/rust-sdk) - Rust MCP SDK
- [tokio](https://tokio.rs/) - Async runtime

## 📄 License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.

---

⭐ If this project helps you, please give us a Star!

## Hosted deployment

A hosted deployment is available on [Fronteir AI](https://fronteir.ai/mcp/adancurusul-serial-mcp-server).

