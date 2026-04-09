# Emu Test Suite

This directory contains the test suite for the Emu project.

## Directory Structure

```
tests/
├── support/        # Shared test infrastructure (factories, builders, assertions)
├── integration/    # Multi-module integration tests
├── performance/    # Speed and memory benchmarks
├── fixtures/       # Fixture-based tests with real command output data
├── unit/           # Remaining unit tests (managers, UI with MockBackend)
├── common/         # Legacy common helpers (being replaced by support/)
└── *.rs            # Standalone test binaries (android, ios, app, ui)
```

## Running Tests

```bash
# Full suite (recommended)
cargo test --features test-utils --bins --tests

# Inline unit tests only
cargo test --lib

# Integration tests (via lib.rs)
cargo test --test lib --features test-utils

# Standalone test binaries
cargo test --test app_state_test
cargo test --test android_manager_test --features test-utils
cargo test --test support_smoke_test

# With output
cargo test -- --nocapture

# Serial execution (avoids env var races)
RUST_TEST_THREADS=1 cargo test --features test-utils
```

## Test Infrastructure (`tests/support/`)

Shared utilities for integration tests:

- **`devices.rs`** — Device factory functions (`android_device()`, `ios_device()`)
- **`state.rs`** — `TestStateBuilder` for decoupled state setup
- **`assertions.rs`** — Predicate-based assertion helpers
- **`managers.rs`** — `MockDeviceManager` factory functions
- **`fixtures.rs`** — JSON fixture loader (runtime `fs::read_to_string`)
- **`contract.rs`** — `DeviceManager` trait contract tests

### Usage

```rust
mod support;
use support::*;

#[test]
fn test_example() {
    let state = TestStateBuilder::new()
        .with_android_devices(vec![android_device("Pixel")])
        .in_mode(Mode::Normal)
        .build();

    assert!(state.is_normal_mode());
    assert_eq!(state.android_device_count(), 1);
}
```

## Test Strategy

### Fixture-Based Approach

All tests use `MockCommandExecutor` to mock external commands — no Android SDK or Xcode required:

```rust
let mock_executor = MockCommandExecutor::new()
    .with_success("avdmanager", &["list", "avd"], "AVD list output")
    .with_success("adb", &["devices"], "List of devices attached\n");
```

### Test Categories

| Category    | Location                     | Description                                       |
| ----------- | ---------------------------- | ------------------------------------------------- |
| Inline unit | `src/**/*.rs` `#[cfg(test)]` | Pure logic (state, models, validation, constants) |
| Integration | `tests/integration/`         | Multi-component workflows                         |
| Standalone  | `tests/*.rs` (root)          | Independent test binaries                         |
| Performance | `tests/performance/`         | Speed and memory benchmarks                       |
| Fixture     | `tests/fixtures/`            | Real command output parsing                       |

## Coverage

```bash
# Generate coverage report
cargo llvm-cov --html --features test-utils
open target/llvm-cov/html/index.html
```

## Troubleshooting

1. **`test-utils` feature not found** — Use `cargo test --features test-utils`
2. **Android SDK errors** — Use `setup_mock_android_sdk()` for mock environment
3. **Parallel execution errors** — Use `RUST_TEST_THREADS=1 cargo test`
