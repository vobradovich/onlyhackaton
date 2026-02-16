# Nak3dCrypto

Nak3dCrypto is an on-chain creator content service built for Gear Protocol with Sails.

- Models create profiles, publish previews, and attach paid encrypted content.
- Fans browse paid content and buy access to a specific item.
- The contract verifies model-issued cryptographic grants, stores buyer-specific encrypted grants, and splits payment between model payout and protocol fee.

The workspace includes:
- `onlyhack`: WASM/IDL build target and integration tests.
- `onlyhack-app`: contract business logic.
- `onlyhack-client`: client for program/test/off-chain interaction.
- `tui`: terminal UI (`ratatui + crossterm`) for local simulation with MODEL/FAN flows.

## Docs

- Architecture and contract API: [ARCH.md](ARCH.md)
- Terminal UI behavior and keybindings: [TUI.md](TUI.md)

## Build

```bash
cargo build --release
```

## Test

```bash
cargo test --release
```

## License

The source code is licensed under the [MIT license](LICENSE).
