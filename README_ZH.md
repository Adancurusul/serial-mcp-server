# 串口 MCP 服务器

[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://rust-lang.org)
[![RMCP](https://img.shields.io/badge/RMCP-0.3.2-blue.svg)](https://github.com/modelcontextprotocol/rust-sdk)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)

专业的模型上下文协议 (MCP) 串口通信服务器。为 AI 助手提供包括嵌入式系统、IoT 设备和硬件调试在内的全面串口通信功能，支持真实硬件集成。

> 📖 **语言版本**: [English](README.md) | [中文](README_ZH.md)

## ✨ 功能特性

- 🚀 **生产就绪**: 真实硬件集成，提供6个综合串口通信工具
- 🔌 **跨平台支持**: Windows、Linux、macOS，自动端口检测
- 📡 **完整串口控制**: 列出端口、连接、发送/接收数据、关闭连接，并设置 RTS/DTR 控制线
- 📝 **多种数据格式**: UTF-8、Hex、二进制编码支持，带超时处理
- 🛠️ **硬件集成**: 经过STM32、Arduino、ESP32等嵌入式系统测试
- 🤖 **AI集成**: 与Claude和其他AI助手完美兼容
- 🧪 **全面测试**: 核心串口数据工具已在真实硬件上验证，控制线参数处理有单元测试覆盖
- ⚡ **高性能**: 基于Tokio异步运行时，支持并发连接

## 🏗️ 架构

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   MCP 客户端    │◄──►│  串口 MCP        │◄──►│  串口设备       │
│   (Claude/AI)   │    │  服务器          │    │  硬件           │
└─────────────────┘    └──────────────────┘    └─────────────────┘
                              │
                              ▼
                       ┌──────────────────┐
                       │  目标设备        │
                       │  (STM32/Arduino) │
                       └──────────────────┘
```

## 🚀 快速开始

### 前置要求

**硬件要求:**
- **串口设备**: STM32, Arduino, ESP32, 或任何UART兼容设备
- **连接**: USB转串口转换器或内置USB-UART
- **USB线**: 用于连接设备到PC

**软件要求:**
- Rust 1.70+ 
- 串口设备驱动程序（大多数系统自动检测）

### 安装

```bash
# 克隆并从源码构建
git clone https://github.com/adancurusul/serial-mcp-server.git
cd serial-mcp-server
cargo build --release
```

### 基本使用

**配置 MCP 客户端**

#### Claude Desktop 配置示例

添加到 Claude Desktop 配置文件:

**Windows 示例:**
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

**macOS/Linux 示例:**
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

其他例如cursor、claude code等参考对应工具文档

## 🎯 试试 STM32 演示

我们提供了一个全面的 **STM32 串口通信演示**，展示了所有功能：

```bash
# 进入示例目录
cd examples/STM32_demo

# 构建并运行固件
cargo run --release

# 与 MCP 服务器配合使用，获得完整的串口通信体验
```

**演示内容:**
- ✅ **交互式串口命令**: 发送命令并获得实时响应
- ✅ **核心 MCP 工具**: 在真实 STM32 硬件上完整验证数据通路
- ✅ **硬件控制**: LED 切换、计数器系统、闪烁模式
- ✅ **命令接口**: 帮助系统与交互式命令处理

[📖 查看 STM32 演示文档 →](examples/STM32_demo/README.md)

### AI 助手使用示例

#### 列出可用串口
```
请列出系统上可用的串口
```

#### 连接串口设备
```
使用波特率115200连接到我的STM32设备的COM19端口
```

#### 发送交互命令
```
发送 'H' 命令获取帮助菜单，然后发送 'L' 切换LED
```

#### 读取设备响应
```
从串口设备读取响应，超时时间2秒
```

#### 完整通信测试
```
请帮我测试COM19上STM32板的所有6个MCP串口工具。先列出端口，然后连接，发送一些命令，读取响应，在适配器支持时设置 DTR/RTS，最后关闭连接。
```

## 🛠️ 完整工具集 (6个工具)

核心数据工具已通过真实 STM32 硬件测试和验证：

### 📡 串口通信 (6个工具)
| 工具 | 描述 | 状态 |
|------|------|------|
| `list_ports` | 发现系统上可用的串口 | ✅ 生产就绪 |
| `open` | 打开带配置的串口连接 | ✅ 生产就绪 |
| `write` | 向连接的串口设备发送数据 | ✅ 生产就绪 |
| `read` | 从串口设备读取数据（带超时） | ✅ 生产就绪 |
| `close` | 清洁地关闭串口连接 | ✅ 生产就绪 |
| `set_control_lines` | 设置已打开连接的 RTS 和/或 DTR 线电平 | ✅ 已实现 |

**✅ 6/6 工具可用 - 核心数据通路已通过真实硬件验证**

## 🌍 支持的硬件

### 串口设备
- **STM32**: 所有带UART功能的STM32系列
- **Arduino**: Uno, Nano, ESP32, ESP8266
- **嵌入式系统**: 任何带UART/USB-Serial接口的设备
- **工业设备**: Modbus, RS485转换器
- **IoT设备**: 带串口通信的传感器、执行器
- **其他**: 任何带UART/USB-Serial接口的设备

### 串口接口
- **USB转串口**: CH340, CH343, FTDI, CP2102
- **内置USB**: 带USB-CDC的STM32, Arduino Leonardo
- **硬件UART**: 直接UART连接

### 平台支持

| 平台 | 端口格式 | 示例 |
|------|---------|------|
| Windows | `COMx` | COM1, COM3, COM19 |
| Linux | `/dev/ttyXXX` | /dev/ttyUSB0, /dev/ttyACM0 |
| macOS | `/dev/tty.xxx` | /dev/tty.usbserial-1234 |

## 🏆 生产状态

### ✅ 完全实现并测试

**当前状态: 生产就绪**

- ✅ **完整的串口集成**: 真实硬件数据通信加 RTS/DTR 控制线操作
- ✅ **硬件验证**: 在STM32 + CH343 USB-Serial COM19端口上测试
- ✅ **交互式通信**: 完整的双向命令/响应系统
- ✅ **多平台支持**: Windows、Linux、macOS支持，自动检测
- ✅ **连接管理**: 强大的连接处理与适当的清理
- ✅ **AI集成**: 完美的MCP协议兼容性

## 📦 技术特性

### 串口实现
- **跨平台**: 自动端口检测和配置
- **多种编码**: UTF-8、Hex、二进制数据支持
- **超时处理**: 可配置的读写超时
- **连接池**: 多个并发串口连接

### 性能特征
- **端口发现**: 快速枚举可用端口
- **连接速度**: 快速建立连接
- **数据吞吐**: 高效数据传输，最小延迟
- **会话稳定性**: 经过长时间运行测试

## 🙏 致谢

感谢以下开源项目：

- [serialport-rs](https://crates.io/crates/serialport) - 串口通信库
- [rmcp](https://github.com/modelcontextprotocol/rust-sdk) - Rust MCP SDK
- [tokio](https://tokio.rs/) - 异步运行时

## 📄 许可证

本项目采用 MIT 许可证。详细信息请参阅 [LICENSE](LICENSE) 文件。

---

⭐ 如果这个项目对你有帮助，请给我们一个 Star！
