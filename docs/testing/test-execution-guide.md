# Emu Test Execution Guide (Developer Guide)

## Overview

This guide explains how to efficiently execute tests in the Emu project.
The Emu test suite is designed to be **emulator/simulator startup-free**,
providing fast execution and stable results.

## Basic Test Execution

### Run All Tests

```bash
# Recommended: Run all tests (excluding doctests)
cargo test --bins --tests

# Or, include doctests
cargo test
```

**Execution time**: ~13-15 seconds (208+ tests)

### Category-based Test Execution

#### Security Tests

```bash
# All security tests
cargo test --test advanced_security_test
cargo test --test path_traversal_security_test
cargo test --test input_validation_comprehensive_test

# Run together
cargo test security
```

#### Performance Tests

```bash
# Startup performance test
cargo test startup_performance_test -- --nocapture

# Panel switching performance
cargo test --test panel_switching_performance_test

# Responsiveness validation
cargo test --test responsiveness_validation_test
```

#### Device Lifecycle Tests

```bash
# Comprehensive lifecycle test
cargo test --test device_lifecycle_comprehensive_test

# Device creation and operation tests
cargo test --test device_operations_status_test
```

#### UI & Navigation Tests

```bash
# Focus and theme tests
cargo test --test ui_focus_and_theme_test

# Device creation navigation
cargo test --test device_creation_navigation_test
```

### Detailed Test Execution

```bash
# Execution with verbose output
cargo test -- --nocapture

# Detailed execution of specific test
cargo test startup_performance_test -- --nocapture --test-threads=1
```

## Test Coverage Measurement

### Using cargo-tarpaulin

```bash
# Install (first time only)
cargo install cargo-tarpaulin

# Measure coverage
cargo tarpaulin --out xml --output-dir coverage --all-features --bins --tests --timeout 120

# Generate HTML report
cargo tarpaulin --out html --output-dir coverage_report --all-features --bins --tests
```

### Current Coverage

- **Overall Coverage**: 14.5%
- **Security Features**: 85%+
- **Device Operations**: 75%+
- **UI Components**: 60%+

**Note**: For TUI applications, 10-20% coverage is considered appropriate.

## Performance Requirements

### Execution Time Standards

| Test Category       | Target Time  | Current Value  |
| ------------------- | ------------ | -------------- |
| All tests           | < 20 seconds | ~14 seconds ✅ |
| Security tests      | < 5 seconds  | ~3 seconds ✅  |
| Startup performance | < 3 seconds  | ~2 seconds ✅  |
| UI tests            | < 8 seconds  | ~6 seconds ✅  |

### Application Performance Standards

| Metric            | Target Value | Measurement Method                 |
| ----------------- | ------------ | ---------------------------------- |
| Startup time      | < 150ms      | `startup_performance_test`         |
| Panel switching   | < 100ms      | `panel_switching_performance_test` |
| Keyboard response | < 8ms        | `responsiveness_validation_test`   |

## CI/CD Execution

### GitHub Actions

Tests are executed in 4 parallel jobs:

1. **Test Coverage**: Coverage measurement and PR comments
2. **Security Tests**: Dedicated security test execution
3. **Performance Tests**: Performance regression detection
4. **Comprehensive Test Suite**: Full test execution

```yaml
# Automatically executed in .github/workflows/test-coverage.yml
- push to main branch
- pull request creation/update
```

### Local CI Environment Reproduction

```bash
# Format check
cargo fmt -- --check

# Clippy check (CI level)
cargo clippy --all-targets --all-features -- -D warnings

# Test execution
cargo test --bins --tests

# Coverage measurement
cargo tarpaulin --out xml --all-features --bins --tests
```

## Troubleshooting

### Common Issues

#### 1. Slow test execution

```bash
# Cause: doctest execution
# Solution: Use --bins --tests flags
cargo test --bins --tests
```

#### 2. Coverage measurement timeout

```bash
# Extend timeout
cargo tarpaulin --timeout 180 --bins --tests
```

#### 3. Concurrent execution conflicts

```bash
# Run single-threaded
cargo test -- --test-threads=1
```

#### 4. Specific test failure

```bash
# Run individual test with details
cargo test test_name -- --nocapture --exact
```

### Debug Environment Variables

```bash
# Rust log level setting
export RUST_LOG=debug

# Detailed output during test execution
export RUST_BACKTRACE=1

# Emu-specific debug (during application execution)
cargo run -- --debug
```

## Best Practices

### Development Flow

1. **Before implementation**: Check related tests
2. **During implementation**: Frequent test execution (`cargo test --bins --tests`)
3. **After completion**: Coverage check and performance validation
4. **Before commit**: Run all checks

```bash
# Recommended pre-commit check
cargo fmt
cargo clippy --all-targets --all-features -- -D warnings
cargo test --bins --tests
```

### Efficient Test Execution

```bash
# For frequent execution during development (fast)
cargo test --bins --tests --quiet

# For detailed confirmation
cargo test specific_test_name -- --nocapture

# For coverage confirmation (weekly)
cargo tarpaulin --out html --output-dir coverage_report
```

### Test Result Interpretation

#### Success example

```
test result: ok. 208 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

#### Handling partial failures

```bash
# Re-run only failed tests
cargo test --bins --tests -- --failed

# Run specific failed test with details
cargo test failing_test_name -- --nocapture
```

## Considerations for New Feature Development

### Security-related Features

- Always add corresponding security tests
- Verify path traversal and command injection protections
- Consider test cases for new attack patterns

### Device Operation Features

- Make error handling tests mandatory
- Verify safety during concurrent processing
- Confirm appropriate handling of resource limitations

### UI Features

- Include keyboard navigation tests
- Verify focus state management
- Confirm behavior on different screen sizes

## References

- [Test Categories Guide](./test-categories.md)
- [Test Addition Guidelines](./test-addition-guidelines.md)
- [Rust Testing Official Documentation](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [cargo-tarpaulin Official Documentation](https://github.com/xd009642/tarpaulin)
