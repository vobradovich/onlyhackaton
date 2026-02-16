## TUI Current State

`tui/` is implemented as a separate workspace member using `ratatui + crossterm`.

- Runtime: local simulation mode only, backed by `gtest`.
- Architecture: split into app state, simulation layer, event loop, and rendering.
- Header: shows `OnlyNak3d TUI` plus role-specific ANSI-styled ASCII avatar.

## Roles

- Two runtime roles: `MODEL` and `FAN`.
- Role switch hotkey: `Tab`.

### MODEL

- Create profile via prompt (`c`):
  - step 1: name
  - step 2: about
- Add paid content via prompt (`a`):
  - step 1: preview
  - step 2: plaintext
  - step 3: price
- Add paid content is allowed only after profile is created.
- Model screen shows:
  - added paid content table (`content id`, `preview`, `price`)
  - fan purchase history table (`buyer`, `content id`, `price`)
- Model balance is displayed at top.

### FAN

- Lists hidden paid content derived from `get_profiles`.
- Buy selected content (`b`) and display decrypted result.
- Fan balance is displayed at top.
- Selection controls: `j/k` or arrow keys.

## Keybindings

- `q`: quit
- `Tab`: switch role
- `r`: refresh profiles and balances
- `Esc`: cancel active prompt
- `Backspace`: edit active prompt input
- `Enter`: submit/advance active prompt

## Persistence

- Fan keypair is generated in-session and persisted to `fan_key.json` (`sk_hex`, `pk_hex`).
