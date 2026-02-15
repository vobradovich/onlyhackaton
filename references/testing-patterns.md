# Testing Patterns

Use these test patterns for Sails programs. Keep guidance repo-agnostic.

## Gtest Harness Pattern

- Use `sails_rs::gtest::System` + `GtestEnv` to spin up a test environment.
- Build a helper like `deploy_with_data(...)` to deploy and seed state.
- Allocate any sharded storage before seeding data.

## Client and Service Usage

- Use the generated client to access services.
- Use `.with_actor_id(...)` to simulate different callers.
- Use event listeners to assert emitted events.

## Assertions

- Prefer small helpers for Result checks (e.g., `assert_ok!`, `assert_err!`) when available.
- When testing panics, compare the gtest panic payload for the exact message.

## Async Tests

- Use `#[tokio::test]` for async service calls.
- Keep tests granular: approvals, transfers, and edge cases (zero, overflow, admin gating).

## Optional: awesome-sails

If available locally, consult:
- `tests/awesome-sails-test/app/tests/gtest.rs`
- `tests/awesome-sails-test/app/tests/common/mod.rs`
- `tests/access-control-test/app/tests/gtest.rs`

