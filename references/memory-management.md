# Memory and Storage Management

Focus on reducing storage pressure and avoiding unsafe updates.

## Sharded Storage

- Sharded storage helps manage map capacity and storage costs.
- Pre-size shards and allocate lazily with `allocate_next_shard()`.
- If capacity is tight, add a controlled admin method to append/allocate shards.

## Non-Zero Storage

- Prefer non-zero wrappers for balances and allowances to avoid storing zero values.
- Zero values often imply removal; design accordingly.

## Balance and Allowance Semantics

- Burns and transfers may remove entries when balances hit zero.
- Allowance decreases should update expiry and remove when allowance reaches zero.
- Infinite allowances should only update expiry on decrease.

## Pausable Storage

- Wrap mutable storage with a pause-aware wrapper when pause semantics are needed.
- `get_mut()` and `replace()` should fail when paused; read-only access should still work.

## Optional: awesome-sails

If available locally, consult:
- `utils/src/map.rs`
- `crates/awesome-sails/vft/utils/src/balances.rs`
- `crates/awesome-sails/vft/utils/src/allowances.rs`
- `utils/src/pause.rs`

