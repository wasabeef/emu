# AGENTS.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Emu is a lazygit-inspired TUI for managing Android emulators and iOS simulators, built with Rust. Async-first architecture with trait-based platform abstraction (`DeviceManager`), centralized state (`AppState` with `Arc<Mutex<>>`), and ratatui-based UI.

See [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) for full architecture details.

## Development Commands

```bash
cargo build                                              # Build
cargo run                                                # Run (default: emu binary)
cargo run -- --debug                                     # Run with debug logging
cargo check                                              # Type check (fast)
cargo clippy --all-targets --all-features -- -D warnings # Lint (CI-level)
cargo fmt                                                # Format
cargo test --bins --tests                                # Run tests (recommended)
cargo test --features test-utils                         # Run all tests including integration
```

## Key Files

| File                          | Role                                             |
| ----------------------------- | ------------------------------------------------ |
| `src/app/mod.rs`              | Main event loop shell and app coordination       |
| `src/app/state/mod.rs`        | `AppState`, `ApiLevelManagementState::is_busy()` |
| `src/managers/android/mod.rs` | Android AVD management, install progress         |
| `src/managers/ios.rs`         | iOS simulator management (macOS only)            |
| `src/managers/common.rs`      | `DeviceManager` trait                            |
| `src/ui/render.rs`            | Three-panel layout rendering                     |
| `src/constants/`              | All constants (NO hardcoded values in source)    |
| `src/models/`                 | Core data structures                             |

## Code Conventions

### String Formatting — CRITICAL

**Always** use inline variable syntax in `format!` and all logging macros:

```rust
// ✅ Correct
format!("Device {name} created")
log::info!("Starting {identifier}")

// ❌ Wrong — clippy::uninlined_format_args error in CI
format!("Device {} created", name)
log::info!("Starting {}", identifier)
```

This applies to `format!`, `println!`, `eprintln!`, `bail!`, `anyhow!`, `log::*`, and test assertions.

### Constants

All hardcoded values must be defined in `src/constants/`. Never use magic numbers or strings in source.

### Error Handling

- `anyhow` for propagation, `thiserror` for custom types
- Never `.unwrap()` or `.expect()` in user-facing code

### Async

- Use `impl Future + Send` for trait methods
- `Arc<Mutex<>>` for shared state; prefer `Arc<AtomicBool>` for simple flags
- Never use `std::sync::Mutex` in async contexts — use `tokio::sync::Mutex`

## Testing

All tests use `MockCommandExecutor` and `MockDeviceManager` — no real Android SDK or Xcode needed.

```bash
cargo test --features test-utils          # Full suite
cargo test -- --nocapture                 # With output
RUST_TEST_THREADS=1 cargo test ...        # Serial (avoids env var races)
```

### Test Infrastructure

- `tests/support/` — shared test foundation (device factories, TestStateBuilder, assertions, contract tests)
- `src/` inline `#[cfg(test)]` — pure logic tests (state, models, validation, constants)
- `tests/integration/` — multi-component integration tests
- `tests/` root — standalone test binaries (android, ios, app, ui)

See [docs/TESTING.md](docs/TESTING.md) for full testing guide.

## Current Status

### Completed

- ✅ Android AVD full lifecycle (create/start/stop/delete/wipe)
- ✅ iOS simulator basic operations (macOS)
- ✅ Three-panel UI with device details
- ✅ Real-time logcat streaming with filtering
- ✅ API level management with install progress UI
- ✅ `ApiLevelManagementState::is_busy()` — centralized busy-check
- ✅ Install progress: 100% shown on completion, stale callbacks prevented
- ✅ Ultra-responsive input (8ms polling, no debouncing)
- ✅ Background loading, smart caching, incremental refresh
- ✅ Comprehensive test suite (720+ tests, CI/CD ready)
- ✅ PostToolUse hook (`.claude/settings.json`): `cargo check` after `.rs` edits

### Known Issues

- Android state detection: occasional inaccuracy in AVD-to-emulator-serial mapping
- iOS device details: limited info compared to Android
- Performance tests: `test_cache_performance` can be flaky on loaded CI runners

## Hooks & CI

- **pre-commit**: `cargo clippy --all-targets --all-features -- -D warnings` + `cargo fmt`
- **pre-push**: `RUST_TEST_THREADS=1 cargo test --bins --tests --features test-utils`
- **PostToolUse** (`.claude/settings.json`): `cargo check` after `.rs` file edits
- **CI**: Build → Check & Lint → Test (ubuntu + macos)
