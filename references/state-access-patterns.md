# State Access Patterns

Use these patterns when structuring state in Sails programs. Keep guidance repo-agnostic.

## Program State Wiring

- Store program state in `RefCell` fields on the `Program` struct.
- Expose storage through `StorageRefCell` adapters for services and helpers.
- For pausable state, wrap `StorageRefCell` with a `Pausable`/`PausableRef` wrapper.

## Access Control

- Use role-based access control when admin gating is needed.
- Grant the initial admin in `new()` using the deployer (`Syscall::message_source()`).
- Expose access control via a method returning an access-control service wrapper.

## Service Exposure Rules

- `#[service]` impls should only include API methods.
- `#[program]` impls should only wire services and lifecycle helpers (like `handle_reply`).
- If a service needs to emit events, prefer `#[service(events = ...)]`.

## Storage Traits (if using awesome-sails utils)

- `Storage` and `StorageMut` are the main access contracts.
- Use `InfallibleStorage` or `InfallibleStorageMut` when storage access cannot fail.
- `StorageRefCell` is the canonical adapter for `RefCell<T>` in program state.

