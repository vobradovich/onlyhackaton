# Sails Templates and Examples

Use these as scaffolding references. Prefer upstream Sails templates/examples if available in the current repo.

## Minimal Program

- Start with `#![no_std]`.
- Use `#[sails_rs::program]` on the `Program` impl.
- Expose services via methods returning service structs.

## Service Shape

- Define a service struct, implement methods under `#[sails_rs::service]`.
- Use `#[export]` for commands and queries.
- Keep logic inside the service; keep program wiring thin.

## Client and WASM Exports

- For programs with client bindings, include a `client` module and `WASM_BINARY` export pattern.
- For simple programs with no services, keep the program empty and focused.

## Optional: upstream Sails repo

If `../sails` is available on this device, consult:
- `templates/program/app/src/lib.rs`
- `templates/program/src/lib.rs`
- `examples/ping-pong-stack/src/lib.rs`
- `examples/no-svcs-prog/app/src/lib.rs`

