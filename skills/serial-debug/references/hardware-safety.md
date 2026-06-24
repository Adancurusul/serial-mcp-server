# Hardware Safety

Serial debugging can change device state. Be explicit about the port, baud rate, and control-line intent before touching hardware.

## Checks Before Writing

- Confirm the selected port name comes from `serial-mcp-server list-ports --json`.
- Confirm voltage compatibility between the adapter and target board. Do not assume 5 V tolerance.
- Confirm the protocol baud rate, data bits, parity, stop bits, and flow control.
- Avoid sending reset, erase, bootloader, or firmware-update commands unless the user requested that operation.

## RTS and DTR

RTS and DTR are often wired to reset or boot mode on development boards. Before using `set-control-lines`, mention that the line change can reset or reconfigure the target if the adapter wiring uses those pins.

Use explicit levels:

```bash
serial-mcp-server set-control-lines --port <port> --rts high --dtr low --json
```

## Evidence Boundaries

If no device is connected, only report CLI availability and port discovery. Mark write/read, probe, and RTS/DTR behavior as manual hardware validation.
