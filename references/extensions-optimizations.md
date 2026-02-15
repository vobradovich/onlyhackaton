# Extensions and Optimizations

Use this when extending services, adding admin flows, or optimizing storage.

## Extension Patterns

- Keep extensions in separate services with clear responsibilities.
- Prefer exposing pagination and maintenance endpoints explicitly (e.g., shard allocation, list methods).
- Emit events for user-visible state changes; keep internal maintenance silent unless it affects clients.

## Admin Patterns

- Gate admin actions with role checks.
- Use distinct roles for mint/burn/pause or other privileged actions.
- Keep admin logic in a dedicated service; wire it in the program layer.

## Storage Optimization

- If using sharded storage, pre-size shards and allocate lazily with `allocate_next_shard()`.
- Avoid unsafe inserts unless you can guarantee uniqueness.
- If capacity is tight, add a controlled admin method to append/allocate shards.

## Performance and Correctness Helpers

- Use early-return helpers (e.g., `ok_if!`, `ensure!`) to enforce invariants.
- Prefer non-zero wrappers for balances/allowances to avoid storing zero values.
- Model infinite allowances explicitly and document their behavior.

## Optional: awesome-sails

If available locally, consult:
- `crates/awesome-sails/vft/src/lib.rs`
- `crates/awesome-sails/vft-extension/src/lib.rs`
- `crates/awesome-sails/vft-admin/src/lib.rs`
- `crates/awesome-sails/vft-native-exchange/src/lib.rs`
- `crates/awesome-sails/vft-native-exchange-admin/src/lib.rs`

