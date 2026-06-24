# STM32 Serial Communication Demo

![STM32G4 Development Board](img/stm32g4.jpg)

This example firmware provides an interactive USART command interface for testing `serial-mcp-server` with real STM32 hardware.

## Hardware Requirements

- STM32 development board with USART1 on PA9/PA10, or equivalent firmware adaptation.
- USB-to-serial adapter such as CH343, FTDI, CP2102, or a board-provided USB-UART.
- Common ground between adapter and target board.
- Voltage-compatible TX/RX levels.
- LED on PB7 if you want the LED commands to show visible output.

## Serial Configuration

| Setting | Value |
| --- | --- |
| Baud rate | 115200 |
| Data bits | 8 |
| Parity | None |
| Stop bits | 1 |
| Flow control | None |

## Build And Run Firmware

```bash
cd examples/STM32_demo
cargo run --release
```

## Firmware Commands

| Command | Behavior |
| --- | --- |
| `H` or `h` | Print help. |
| `L` or `l` | Toggle LED state. |
| `C` or `c` | Print and increment counter. |
| `R` or `r` | Reset counter to zero. |
| `B` or `b` | Blink LED three times. |
| Other input | Echo the character. |

## CLI Smoke

From the repository root:

```bash
serial-mcp-server list-ports --json
serial-mcp-server probe --port <port> --baud 115200 --json
serial-mcp-server write --port <port> --baud 115200 --data H --read --timeout-ms 2000 --json
```

Use the port name reported by `list-ports`. On Windows this may look like `COM19`; on Linux or macOS it will usually be under `/dev`.

## MCP Smoke

Start the MCP server:

```bash
serial-mcp-server serve
```

Use the MCP client to run this sequence:

1. `list_ports`
2. `open` with baud rate `115200`
3. `write` command `H`
4. `read` with a timeout
5. `set_control_lines` only if you intentionally want RTS/DTR changes
6. `close`

## Control-Line Caution

RTS and DTR behavior depends on adapter and board wiring. These lines can reset a board or put it into boot mode. Treat a successful `set_control_lines` response as software request evidence, not as electrical measurement evidence.

## Evidence Boundaries

The historical COM19 test documentation in `docs/serial-mcp-testing-documentation.md` records one hardware setup. New release claims should be based on current command output from the device under test.
