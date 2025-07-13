# Test Addition Guidelines

## Overview

This document explains the procedures and best practices for adding new tests to the Emu project.
By creating high-quality test code, we maintain project stability and maintainability.

## Basic Principles

### 1. Emulator Independence Principle

- **Mandatory**: All tests must be implemented without emulator/simulator startup
- **Reason**: Fast execution, CI/CD stability, improved development efficiency
- **Implementation**: Use mock data and stubs

### 2. Clear Responsibility Scope

- **Unit Tests**: Verify behavior of single functions/methods
- **Integration Tests**: Confirm coordination between multiple components
- **Security Tests**: Verify vulnerabilities and attack resistance

### 3. Maintainability Focus

- Understandable test names
- Clear assertions
- Appropriate documentation

## Test Addition Procedure

### Step 1: Category Determination

Select the appropriate test category for new features:

```
1. Security Tests
   - Command execution, file operations, input validation related

2. Device Lifecycle Tests
   - Device creation, operations, deletion, state management related

3. Performance Tests
   - Response time, memory usage, UI performance related

4. UI & Navigation Tests
   - User interface, keyboard operations related

5. Integration Tests
   - Multi-feature coordination, end-to-end workflows

6. Specialized Tests
   - Other specific features
```

### Step 2: File Structure Decision

#### Creating New Test Files

```bash
# For security tests
tests/security_[feature_name]_test.rs

# For device feature tests
tests/device_[feature_name]_test.rs

# For UI tests
tests/ui_[feature_name]_test.rs

# For performance tests
tests/[feature_name]_performance_test.rs
```

#### Adding to Existing Files

If there are related features in existing test files, consider adding there.

### Step 3: Test File Templates

#### Basic Template

```rust
//! [Feature Name] Tests
//!
//! [Brief feature description and test purpose]

use std::sync::Arc;
use tokio::sync::Mutex;

// Required imports
use emu::app::state::AppState;
use emu::models::device::{AndroidDevice, DeviceStatus};
// Other necessary modules

/// Create test application state
async fn create_test_app_state() -> Arc<Mutex<AppState>> {
    Arc::new(Mutex::new(AppState::new().await))
}

/// Create test sample device
fn create_test_android_device(name: &str) -> AndroidDevice {
    AndroidDevice {
        name: name.to_string(),
        path: format!("/path/to/{name}"),
        status: DeviceStatus::Stopped,
        api_level: 30,
        target: "default".to_string(),
        cpu_arch: "x86_64".to_string(),
        ram_mb: 2048,
        storage_mb: 8192,
    }
}

#[tokio::test]
async fn test_[feature_name]_basic_functionality() {
    // Test implementation
    let app_state = create_test_app_state().await;

    // Test logic
    assert!(true); // Appropriate assertion
}

#[tokio::test]
async fn test_[feature_name]_error_handling() {
    // Error case testing
}

#[test]
fn test_[feature_name]_edge_cases() {
    // Edge case testing (for non-async cases)
}
```

#### Security Test Specific Template

```rust
//! [Security Feature] Security Tests
//!
//! [Attack patterns and defense feature description]

use emu::utils::validation::{DeviceNameValidator, Validator};

/// Dangerous input pattern definitions
const MALICIOUS_PATTERNS: &[&str] = &[
    "normal_input",
    "; rm -rf /",
    "$(malicious_command)",
    "../../../etc/passwd",
    // Other attack patterns
];

#[test]
fn test_input_validation_against_injection() {
    let validator = DeviceNameValidator::new();

    for pattern in MALICIOUS_PATTERNS {
        let result = validator.validate(pattern);
        assert!(
            !result.is_valid || is_safe_input(pattern),
            "Dangerous pattern should be rejected: {pattern}"
        );
    }
}

/// Determine if input is safe
fn is_safe_input(input: &str) -> bool {
    // Security check implementation
    !input.contains(';') && !input.contains("$(")
}
```

## Quality Standards

### Test Naming Conventions

#### ✅ Good Examples

```rust
#[test]
fn test_device_creation_with_invalid_name_should_fail() {}

#[test]
fn test_command_injection_patterns_are_rejected() {}

#[test]
fn test_panel_switching_performance_under_150ms() {}
```

#### ❌ Bad Examples

```rust
#[test]
fn test1() {}

#[test]
fn test_device() {}

#[test]
fn check_stuff() {}
```

### Assertion Quality

#### ✅ Good Examples

```rust
#[test]
fn test_device_name_validation() {
    let validator = DeviceNameValidator::new();
    let result = validator.validate("test-device");

    assert!(result.is_valid, "Valid device name should pass validation");
    assert!(result.errors.is_empty(), "No errors expected for valid input");
    assert_eq!(result.value, "test-device", "Value should be preserved");
}
```

#### ❌ Bad Examples

```rust
#[test]
fn test_device_name_validation() {
    let validator = DeviceNameValidator::new();
    let result = validator.validate("test-device");

    assert!(result.is_valid); // No message
    assert!(true); // Meaningless assertion
}
```

### Documentation Quality

#### File-level Comments

````rust
//! Android Device Creation Security Tests
//!
//! This test module verifies the effectiveness of input validation and
//! security measures during Android device creation.
//!
//! ## Covered Security Risks
//! - Command injection
//! - Path traversal
//! - Input length limitation bypass
//!
//! ## Test Execution Command
//! ```bash
//! cargo test --test android_device_creation_security_test
//! ```
````

#### Test-level Comments

```rust
/// Prevent SQL injection attacks with malicious device names
///
/// # Test Cases
/// - `'; DROP TABLE devices; --` - SQL injection
/// - `" OR "1"="1` - Authentication bypass attempt
/// - `\x00` - Null byte attack
#[test]
fn test_malicious_device_names_are_rejected() {
    // Test implementation
}
```

## Category-specific Detailed Guidelines

### Security Tests

#### Required Test Patterns

1. **Basic Attack Patterns**

   ```rust
   const BASIC_ATTACKS: &[&str] = &[
       "; rm -rf /",
       "$(echo vulnerable)",
       "../../../etc/passwd",
       "<script>alert(1)</script>",
   ];
   ```

2. **Advanced Attack Patterns**

   ```rust
   const ADVANCED_ATTACKS: &[&str] = &[
       "device${IFS}&&${IFS}malicious",
       "%2e%2e%2f%2e%2e%2f%2e%2e%2fetc%2fpasswd",
       "\uff0e\uff0e\u2215\uff0e\uff0e\u2215etc\u2215passwd",
   ];
   ```

3. **Binary & Control Character Attacks**
   ```rust
   const BINARY_ATTACKS: &[&str] = &[
       "\x00", "\x1f", "\x7f",
       "test\r\nmalicious",
       "test\u{202e}gnissecorp", // RTL character
   ];
   ```

#### Security Test Implementation Example

```rust
#[test]
fn test_comprehensive_command_injection_prevention() {
    let patterns = [
        ("basic_semicolon", "; rm -rf /"),
        ("command_substitution", "$(cat /etc/passwd)"),
        ("backtick_execution", "`id`"),
        ("pipe_chain", "| cat /etc/shadow"),
        ("logical_and", "&& malicious_command"),
        ("logical_or", "|| evil_script"),
    ];

    for (test_name, malicious_input) in patterns {
        let is_safe = validate_device_name(malicious_input);
        assert!(
            !is_safe,
            "Test '{test_name}' failed: '{malicious_input}' should be rejected"
        );
    }
}
```

### Device Lifecycle Tests

#### Required Test Scenarios

1. **Normal Path**
   - Device creation → start → stop → deletion
2. **Error Path**
   - Creation attempts with invalid configuration values
   - Operation attempts on non-existent devices
   - Behavior during resource exhaustion

3. **Concurrent Operations**
   - Simultaneous access control
   - Race condition prevention

#### Implementation Example

```rust
#[tokio::test]
async fn test_device_lifecycle_with_resource_constraints() {
    let app_state = create_test_app_state().await;

    // Simulate resource limitations
    let high_resource_device = AndroidDevice {
        name: "resource-intensive".to_string(),
        ram_mb: 16384, // 16GB RAM
        storage_mb: 65536, // 64GB Storage
        ..create_test_android_device("test")
    };

    let result = app_state.lock().await
        .create_android_device(high_resource_device).await;

    // Verify appropriate handling based on resource limitations
    match result {
        Ok(_) => {
            // When system resources are sufficient
            assert!(check_system_resources());
        }
        Err(e) => {
            // Appropriate error handling during resource shortage
            assert!(e.to_string().contains("insufficient resources"));
        }
    }
}
```

### Performance Tests

#### Measurement Target Indicators

1. **Response Time**
   - Startup time: < 150ms
   - Panel switching: < 100ms
   - Keyboard response: < 8ms

2. **Resource Usage**
   - Memory consumption
   - CPU usage

#### Implementation Example

```rust
#[tokio::test]
async fn test_panel_switching_performance() {
    let app_state = create_test_app_state().await;

    // Run benchmark
    let start = std::time::Instant::now();

    for _ in 0..100 {
        app_state.lock().await.switch_to_android_panel();
        app_state.lock().await.switch_to_ios_panel();
    }

    let duration = start.elapsed();
    let avg_switch_time = duration / 200; // 200 switches

    assert!(
        avg_switch_time.as_millis() < 100,
        "Panel switching took {avg_switch_time:?}, should be < 100ms"
    );
}
```

### UI & Navigation Tests

#### Required Test Cases

1. **Keyboard Navigation**
   - Tab key focus movement
   - Circular navigation
   - Shortcut keys

2. **Screen State Management**
   - Focus state retention
   - Modal display/hide
   - Error message display

#### Implementation Example

```rust
#[test]
fn test_circular_navigation_in_device_creation_form() {
    let mut form_state = DeviceCreationFormState::new();

    // Moving from last field to next returns to first
    form_state.focus_field(CreateDeviceField::Storage);
    form_state.focus_next_field();

    assert_eq!(
        form_state.current_field(),
        CreateDeviceField::DeviceName,
        "Should cycle back to first field"
    );
}
```

## Code Review Standards

### Review Checklist

#### ✅ Mandatory Check Items

- [ ] Test name clearly describes functionality
- [ ] Executable without emulator/simulator startup
- [ ] Includes appropriate assertion messages
- [ ] Error cases are tested
- [ ] Documentation comments are properly written
- [ ] No duplication with existing tests
- [ ] Meets performance requirements

#### Security Test Specific

- [ ] Comprehensively covers related attack patterns
- [ ] Considers risks of false positives/negatives
- [ ] Addresses new vulnerability patterns

#### Performance Test Specific

- [ ] Clear performance standards are set
- [ ] Measurement accuracy and reproducibility are ensured
- [ ] Resource usage is monitored

### Review Comment Examples

#### ✅ Good Review Comments

```
This test is comprehensive and good, but please improve the following points:

1. Add assertion messages
   - `assert!(result.is_valid)` → `assert!(result.is_valid, "Valid input should pass")`

2. Add edge cases
   - Please add test case for empty string

3. Improve documentation
   - Clarify what the test is verifying with comments
```

#### ❌ Bad Review Comments

```
Not enough tests. Add more.
```

## CI/CD Integration

### Verify New Test Automatic Execution

After adding tests, verify the following:

```bash
# Local operation verification
cargo test --test [new_test_file] --verbose

# Operation verification with all tests
cargo test --bins --tests

# Coverage impact verification
cargo tarpaulin --out xml --bins --tests
```

### GitHub Actions Automatic Execution

New test files are automatically included in the following workflows:

- **test-coverage**: Coverage measurement
- **security-tests**: Security tests (when applicable)
- **performance-tests**: Performance tests (when applicable)
- **comprehensive-test-suite**: Full test execution

## References and Best Practices

### Recommended Resources

- [Rust Testing Guide](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [TDD Best Practices](https://martinfowler.com/bliki/TestDrivenDevelopment.html)
- [OWASP Testing Guide](https://owasp.org/www-project-web-security-testing-guide/)

### Code Sample Collection

- [Security Test Implementation Examples](../tests/advanced_security_test.rs)
- [Lifecycle Test Implementation Examples](../tests/device_lifecycle_comprehensive_test.rs)
- [Performance Test Implementation Examples](../tests/startup_performance_test.rs)

### Team Development Operations

1. **Consultation Before Test Addition**
   - Discuss with team before large test additions
   - Verify duplication with existing tests

2. **Incremental Implementation**
   - Basic feature tests → error cases → edge cases
   - Add incrementally with small pull requests

3. **Continuous Improvement**
   - Regular analysis of test results
   - Continuous addition of new attack patterns
   - Review of performance standards

Please create tests that contribute to improving the quality of the Emu project by following these guidelines.
