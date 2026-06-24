# Troubleshooting

## No Ports Found

Run:

```bash
serial-mcp-server list-ports --json
```

Check cable, adapter driver, permissions, and whether another tool already owns the port.

## Port Open Fails

- Verify the port name exactly matches discovery output.
- Close serial monitors, flashing tools, or old MCP sessions using the same port.
- Try the known protocol baud rate instead of the default.
- On Unix-like systems, check device permissions for `/dev/tty*` or `/dev/cu*`.

## Read Times Out

- Confirm the target firmware actually writes serial output.
- Increase `--timeout-ms`.
- Check line endings and whether the device expects a command terminator.
- For request/response protocols, write first with `--read`.

## Data Looks Wrong

- Recheck `--format`.
- Use `--format hex` when bytes are not valid UTF-8.
- Confirm baud, parity, stop bits, and flow control.

## Control Lines Do Not Behave As Expected

- Confirm adapter wiring for RTS and DTR.
- Check whether the target inverts or routes the lines through reset or boot circuitry.
- Treat the CLI response as requested state evidence, not as electrical measurement evidence.
