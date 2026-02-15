---
name: sails-idiomatic-dev
description: "Idiomatic development for Sails programs and services (Rust), regardless of repository. Use when designing program/service structure, state access, storage wrappers, access control, extensions, performance/optimization choices, testing patterns, or memory management for Sails-based programs."
---

# Sails Idiomatic Dev

Act as an idiomatic Sails guide for any repo. Prefer upstream Sails templates and examples when available. If `awesome-sails` is present on this device, its patterns are optional references for advanced services.

## Quick Start

- Identify whether the user is building a basic program, a service-focused program, or extending existing services.
- Choose state storage patterns that match the use case (see `references/state-access-patterns.md`).
- For extensions/optimizations and shard sizing, consult `references/extensions-optimizations.md`.
- For tests, follow the harness patterns in `references/testing-patterns.md`.
- For memory/storage constraints and non-zero math types, see `references/memory-management.md`.
- For minimal scaffolding, see `references/sails-templates.md`.

## Default Patterns

- Keep program state in `RefCell` fields and expose storage via `StorageRefCell` + `PausableRef` when needed.
- Use role-based access control when admin gating is needed; grant the initial admin in `new()`.
- Prefer `#[service]` for API surfaces, keep `#[program]` for wiring and exposure only.
- Keep storage access transactional: call `get_mut()` only when you are ready to mutate.

## Optional: awesome-sails

If `awesome-sails` is available locally (e.g., `/Users/ukintvs/Documents/projects/awesome-sails`), you may reference its VFT/admin/access-control patterns as optional examples. Do not assume those crates exist in the target repo.

## References Map

- State access patterns: `references/state-access-patterns.md`
- Extensions and optimizations: `references/extensions-optimizations.md`
- Testing patterns: `references/testing-patterns.md`
- Memory management: `references/memory-management.md`
- Sails templates/examples: `references/sails-templates.md`
