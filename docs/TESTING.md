# Testing Guide for Emu

## Overview

Emu uses a comprehensive testing infrastructure based on mocks and fixtures, allowing tests to run without requiring Android SDK or Xcode installations. This approach ensures fast, reliable tests that work consistently in CI/CD environments.

## Quick Start

```bash
# Run all tests (recommended)
cargo test --features test-utils

# Run with output for debugging
cargo test -- --nocapture

# Run specific test categories
cargo test --test unit
cargo test --test integration
cargo test --test performance

# Measure test coverage
cargo llvm-cov --lcov --output-path coverage/lcov.info --features test-utils \
  --ignore-filename-regex '(tests/|src/main\.rs|src/bin/|src/app/test_helpers\.rs|src/fixtures/|src/managers/mock\.rs)'
```

## Test Infrastructure

### MockCommandExecutor

All external command execution is mocked using `MockCommandExecutor`:

```rust
use emu::utils::command_executor::mock::MockCommandExecutor;
use std::sync::Arc;

let mock_executor = MockCommandExecutor::new()
    .with_success("avdmanager", &["list", "avd"], "device list output")
    .with_success("adb", &["devices"], "List of devices attached\n")
    .with_error("emulator", &["-avd", "failing_device"], "Error: Device not found");

let manager = AndroidManager::with_executor(Arc::new(mock_executor)).unwrap();
```

### Mock Android SDK Setup

For tests requiring Android SDK environment:

```rust
use common::setup_mock_android_sdk;

let _temp_dir = setup_mock_android_sdk();
std::env::set_var("ANDROID_HOME", _temp_dir.path());

// Keep temp directory alive during test
std::mem::forget(_temp_dir);
```

### MockBackend for UI Testing

UI components can be tested without a real terminal:

```rust
use ratatui::backend::MockBackend;
use ratatui::Terminal;

let backend = MockBackend::new(120, 40); // width x height
let mut terminal = Terminal::new(backend).unwrap();

// Render UI and verify output
terminal.draw(|f| {
    render_device_list(f, f.size(), &devices);
}).unwrap();

// Assert text appears in output
assert!(backend.contains_text("Device Name"));
```

## Test Categories

### Unit Tests (`tests/unit/`)

Test individual components in isolation:

- **Models**: Device, Error, Platform validation
- **Utils**: Command execution, validation helpers
- **Managers**: Android/iOS manager methods with mocked commands

Example:

```rust
#[test]
fn test_device_validation() {
    let device = AndroidDevice::new("Test_Device");
    assert!(device.validate_name().is_ok());
}
```

### Integration Tests (`tests/integration/`)

Test interactions between multiple components:

- **Device Lifecycle**: Create → Start → Stop → Delete workflows
- **State Management**: App state updates and transitions
- **Event Processing**: User input handling and UI updates
- **Background Tasks**: Concurrent operations and cancellation

Example:

```rust
#[tokio::test]
async fn test_device_creation_workflow() {
    let app_state = setup_test_app_state().await;

    // Test complete creation flow
    app_state.lock().await.show_create_device_dialog();
    // ... test form filling and submission
}
```

### Performance Tests (`tests/performance/`)

Validate performance requirements:

- **Startup Time**: < 150ms
- **Panel Switching**: < 100ms
- **Device Navigation**: < 50ms
- **Log Streaming**: < 10ms latency

Example:

```rust
#[tokio::test]
async fn test_startup_performance() {
    let start = Instant::now();
    let _app = App::new().await;
    let duration = start.elapsed();

    assert!(duration < Duration::from_millis(150));
}
```

### Fixture-Based Tests (`tests/fixtures/`)

Use real command outputs for realistic testing:

```rust
#[test]
fn test_parse_real_avd_output() {
    let fixture = load_fixture("android_avd_list.txt");
    let devices = parse_avd_list(&fixture);

    assert_eq!(devices.len(), 3);
    verify_device_properties(&devices);
}
```

## Writing New Tests

### 1. Choose the Right Category

- **Unit Test**: Testing a single function or struct method
- **Integration Test**: Testing feature workflows
- **Performance Test**: Measuring execution time or resource usage
- **Fixture Test**: Testing parsers with real data

### 2. Use Test Helpers

```rust
// Common test setup
use common::{setup_mock_android_sdk, create_test_app_state};

// Create pre-configured managers
let android_manager = create_mock_android_manager();
let ios_manager = create_mock_ios_manager();
```

### 3. Follow Naming Conventions

- Test functions: `test_<component>_<behavior>`
- Test files: `<module>_test.rs`
- Helper functions: `create_<type>`, `setup_<component>`

### 4. Test Error Cases

Always test both success and failure paths:

```rust
#[test]
fn test_device_creation_success() {
    // Test happy path
}

#[test]
fn test_device_creation_invalid_name() {
    // Test validation failure
}

#[test]
fn test_device_creation_command_failure() {
    // Test external command failure
}
```

## CI/CD Integration

Tests run automatically on:

- Every push to main branch
- All pull requests
- Multiple platforms (Ubuntu, macOS)

### CI Requirements

- All tests must pass
- No compiler warnings (`cargo clippy`)
- Code formatted (`cargo fmt`)
- Test coverage maintained or improved

## Common Issues and Solutions

### Issue: "Tool 'emulator' not found in Android SDK"

**Solution**: Ensure all tests use mock SDK setup:

```rust
let _temp_dir = setup_mock_android_sdk();
std::env::set_var("ANDROID_HOME", _temp_dir.path());
std::mem::forget(_temp_dir);
```

### Issue: Flaky async tests

**Solution**: Use proper async test setup:

```rust
#[tokio::test]
async fn test_async_operation() {
    // Use tokio runtime
}
```

### Issue: Test isolation failures

**Solution**: Create fresh state for each test:

```rust
#[test]
fn test_isolated_operation() {
    let state = AppState::new(); // Fresh state
    // Test operations
}
```

## Test Coverage

### Current Status

- **Coverage**: 47.71% (5,173/10,842 lines)
- **Test Files**: 22+
- **Test Functions**: 180+

### Measuring Coverage

```bash
# Generate coverage report
cargo llvm-cov --html --features test-utils

# View report
open target/llvm-cov/html/index.html
```

### Coverage Goals

- Minimum 70% overall coverage
- Critical paths: 90%+ coverage
- New features: Must include tests

## Best Practices

1. **Test Independence**: Each test should be runnable in isolation
2. **Clear Assertions**: Test one behavior per test function
3. **Descriptive Names**: Test names should explain what they verify
4. **Fast Execution**: Keep individual tests under 1 second
5. **Deterministic**: Same input always produces same output
6. **No Real I/O**: Always use mocks for external dependencies

## Advanced Testing

### Custom Matchers

```rust
// Custom assertion helpers
fn assert_device_running(device: &AndroidDevice) {
    assert_eq!(device.status, DeviceStatus::Running);
    assert!(device.pid.is_some());
}
```

### Test Scenarios

```rust
// Parameterized tests
#[test]
fn test_api_levels() {
    for api in 30..=34 {
        test_device_with_api_level(api);
    }
}
```

### Benchmark Tests

```rust
#[bench]
fn bench_device_list_parsing(b: &mut Bencher) {
    let fixture = load_large_device_list();
    b.iter(|| parse_device_list(&fixture));
}
```

## Future Improvements

1. **Fixture Auto-Update**: Automatically collect real command outputs
2. **Property-Based Testing**: Generate test cases automatically
3. **Mutation Testing**: Verify test quality
4. **Visual Regression Testing**: UI screenshot comparisons
5. **Load Testing**: Simulate many concurrent devices

## Related Documentation

- [ARCHITECTURE.md](ARCHITECTURE.md) - System design and components
- [DEVELOPMENT.md](DEVELOPMENT.md) - Development workflow
- [CLAUDE.md](../CLAUDE.md) - AI assistant guidelines
