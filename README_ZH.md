# 串口 MCP 服务器

[![Rust](https://img.shields.io/badge/rust-1.74+-orange.svg)](https://rust-lang.org)
[![RMCP](https://img.shields.io/badge/RMCP-0.3.2-blue.svg)](https://github.com/modelcontextprotocol/rust-sdk)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)

`serial-mcp-server` 为 AI 工作流提供串口访问能力，包含两种使用方式：

- 面向 MCP 客户端的 stdio MCP 服务器。
- 面向脚本、CI 和 agent skill 的直接 CLI，不需要先配置 MCP。

当前发布目标：`0.3.0`。

语言版本：[English](README.md) | [中文](README_ZH.md)

## 0.3.0 更新简报

- 新增 JSON Macro DSL，用于可复用的串口自动化流程。
- 新增 CLI macro 命令：`macro validate`、`macro list`、`macro plan`、`macro run`。
- 新增 MCP macro 工具，支持运行时内存加载、列出、卸载、规划和运行 macro pack。
- 新增无硬件 simulation，用于 macro 验证、规划和 executor smoke test。
- 不提供独立 Quick API。Quick 风格用法应表达为 macro。

## 环境要求

- Rust 1.74 或更新版本。
- 执行硬件操作时需要串口设备或 USB 转串口适配器。
- 系统需要具备对应串口驱动和端口访问权限。

## 从源码安装

```bash
git clone https://github.com/adancurusul/serial-mcp-server.git
cd serial-mcp-server
cargo build --release
```

构建后的二进制文件位于：

```bash
target/release/serial-mcp-server
```

如果要从当前 checkout 安装到 `PATH`：

```bash
cargo install --path . --locked
```

## CLI 使用

如果不想配置 MCP 客户端，可以直接使用 CLI。

```bash
serial-mcp-server --help
serial-mcp-server list-ports --json
serial-mcp-server probe --port <port> --baud 115200 --json
serial-mcp-server write --port <port> --baud 115200 --data H --read --timeout-ms 1000 --json
serial-mcp-server read --port <port> --baud 115200 --timeout-ms 1000 --json
serial-mcp-server set-control-lines --port <port> --rts high --dtr low --json
```

Macro 自动化命令：

```bash
serial-mcp-server macro validate --file examples/macros/ping.json --json
serial-mcp-server macro list --file examples/macros/ping.json --json
serial-mcp-server macro plan --file examples/macros/ping.json --macro ping --json
serial-mcp-server macro run --file examples/macros/ping.json --macro ping --dry-run --json
serial-mcp-server macro run --file examples/macros/ping.json --macro ping --simulate-read PONG --json
serial-mcp-server macro run --file examples/macros/ping.json --macro ping --port <port> --baud 115200 --json
```

Macro pack 是 `schema_version` 为 `0.3` 的 JSON 文件。v0.3 支持 macro 内的 `send`、`delay`、`expect` 步骤，以及按名称调用 macro 的 assembly。`expect` 支持 `contains` 和 `equals`。

Macro DSL 是受限 DSL，不执行 shell、JavaScript、Python、文件操作、循环、变量、if/else、Quick 命令或 RTS/DTR macro 步骤。

配置命令：

```bash
serial-mcp-server generate-config
serial-mcp-server validate-config --config serial-mcp.toml
serial-mcp-server show-config --config serial-mcp.toml
```

CLI 输出规则：

- stdout 只输出命令数据和 JSON。
- stderr 用于诊断信息。
- `--json` 输出应能被 `jq` 等工具解析。
- 非零退出码表示命令失败。

CLI 支持的数据格式是 `utf8`、`hex` 和 `base64`。二进制载荷请使用 `hex` 或 `base64`。

## MCP 使用

当客户端支持 MCP 工具，并且你希望启动长运行的 stdio server 时使用 MCP。

推荐的服务命令：

```bash
serial-mcp-server serve
```

无子命令启动仍保留为兼容路径；新配置建议显式使用 `serve`。

Claude Desktop macOS/Linux 示例：

```json
{
  "mcpServers": {
    "serial": {
      "command": "/path/to/serial-mcp-server/target/release/serial-mcp-server",
      "args": ["serve"],
      "env": {
        "RUST_LOG": "info"
      }
    }
  }
}
```

Windows 示例：

```json
{
  "mcpServers": {
    "serial": {
      "command": "C:\\path\\to\\serial-mcp-server\\target\\release\\serial-mcp-server.exe",
      "args": ["serve"],
      "env": {
        "RUST_LOG": "info"
      }
    }
  }
}
```

## MCP 工具

| 工具 | 用途 |
| --- | --- |
| `list_ports` | 发现可用串口。 |
| `open` | 打开串口连接。 |
| `write` | 向已打开连接写入 UTF-8、hex 或 base64 数据。 |
| `read` | 从已打开连接读取数据并处理超时。 |
| `close` | 关闭已打开连接。 |
| `set_control_lines` | 设置已打开连接的 RTS 和/或 DTR。 |
| `macro_load` | 验证并把 inline macro pack 或 pack 文件路径加载到服务器内存 registry。 |
| `macro_list` | 列出已加载的 macro pack、macro 和 assembly。 |
| `macro_unload` | 从内存 registry 移除已加载的 macro pack。 |
| `macro_plan` | 无硬件展开已加载 macro 或 assembly。 |
| `macro_run` | 基于已有连接或显式 simulation 输入运行已加载 macro 或 assembly。 |
| `macro_run_inline` | 不存入 registry，直接验证、规划并运行 inline macro pack。 |

MCP macro registry 只在运行时保存在内存中。服务器重启会清空已加载 pack，服务器不会写入持久 macro 库。

## Agent Skill

仓库内包含兼容 Claude Code 和 Codex 的 skill：

```text
skills/serial-debug/
```

该 skill 优先使用 CLI，并把 MCP 作为可选的已配置路径。适用于 agent 列出串口、运行串口 smoke test、运行 macro 自动化、控制 RTS/DTR 或排查 UART/USB 串口问题。

本地开发时，可以把 skill 复制到 agent 的 skill 根目录：

```bash
mkdir -p ~/.codex/skills ~/.claude/skills ~/.agents/skills
cp -R skills/serial-debug ~/.codex/skills/
cp -R skills/serial-debug ~/.claude/skills/
cp -R skills/serial-debug ~/.agents/skills/
```

已测试的显式触发方式：

```text
Codex: Use $serial-debug
Claude Code: /serial-debug
```

本地测试中，Claude Code `--bare` 模式没有解析 `/serial-debug`；skill 触发 smoke test 请使用普通 Claude Code print 或交互模式。

## 硬件安全

串口命令会影响真实硬件。

- 先通过 `serial-mcp-server list-ports --json` 确认端口。
- 连接适配器前确认目标板电平兼容。
- 写入前确认波特率、数据位、校验位、停止位和流控。
- 谨慎操作 RTS 和 DTR。很多开发板会把这些线连接到 reset 或 boot mode。
- 只有命令确实在已连接设备上运行后，才能声称 write/read 或 RTS/DTR 验证通过。

## STM32 示例

STM32 示例位于：

```text
examples/STM32_demo/
```

它提供一个交互式串口命令固件。接线、固件命令、MCP 使用和 CLI smoke 命令见 [examples/STM32_demo/README.md](examples/STM32_demo/README.md)。

## 质量门

发布工作使用仓库内的 `Cargo.lock`，并执行以下质量门：

```bash
cargo fmt --all -- --check
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo test --locked --all-targets --all-features
cargo doc --locked --all-features --no-deps
```

CLI smoke：

```bash
cargo run --locked -- --help
cargo run --locked -- list-ports --json
cargo run --locked -- write --help
cargo run --locked -- set-control-lines --help
cargo run --locked -- macro validate --file examples/macros/ping.json --json
cargo run --locked -- macro run --file examples/macros/ping.json --macro ping --simulate-read PONG --json
```

## 许可证

本项目采用 MIT 许可证。详情见 [LICENSE](LICENSE)。
