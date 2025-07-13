# Emu Test Categories Guide

## Overview

The Emu project test suite is organized into **6 main categories** that comprehensively cover
functionality, security, and performance. All tests are designed to be **emulator/simulator startup-free**,
providing fast and stable execution.

## Test Category Details

### 1. Security Tests

**Purpose**: Prevention of security vulnerabilities and verification of attack resistance

#### 1.1 Advanced Command Injection Tests

**File**: `tests/advanced_security_test.rs`  
**Test Count**: 15  
**Run Command**: `cargo test --test advanced_security_test`

**Coverage**:

- Multi-stage command execution attacks (`;`, `&&`, `||`)
- Command substitution attacks (`$()`, `` ` ` ``)
- Environment variable exploitation (`${IFS}`, `$PATH`)
- Binary data injection
- Resource exhaustion attacks (`/dev/zero`, `:(){ :|:& };:`)
- Privilege escalation attempts (`sudo`, `su`)

**Implementation Example**:

```rust
#[test]
fn test_command_substitution_patterns() {
    let advanced_patterns = vec![
        "device$(echo ';rm -rf /tmp')",        // Command substitution
        "device${IFS}&&${IFS}malicious",       // IFS variable exploitation
        "device`echo 'pwned'`",                // Backtick execution
    ];
    // Validation logic
}
```

#### 1.2 Path Traversal Protection Tests

**File**: `tests/path_traversal_security_test.rs`  
**Test Count**: 14  
**Run Command**: `cargo test --test path_traversal_security_test`

**Coverage**:

- Basic directory traversal (`../`)
- URL encoding attacks (`%2e%2e%2f`)
- Unicode normalization attacks (`\uff0e\uff0e\u2215`)
- Double encoding attacks
- Platform-specific path attacks (Windows `\`, Unix `/`)
- Long path attacks and symlink exploitation

**Implementation Example**:

```rust
fn is_safe_path(path: &str) -> bool {
    if path.contains("..") { return false; }
    if path.contains("%2e%2e") { return false; }
    if path.starts_with('/') || path.contains('\\') { return false; }
    true
}
```

#### 1.3 Comprehensive Input Validation Tests

**File**: `tests/input_validation_comprehensive_test.rs`  
**Test Count**: 14  
**Run Command**: `cargo test --test input_validation_comprehensive_test`

**Coverage**:

- SQL injection (`' OR '1'='1`)
- XSS attacks (`<script>alert(1)</script>`)
- LDAP injection (`*)(uid=*`)
- NoSQL injection (`{"$ne":null}`)
- Internationalization-related attacks (emojis, RTL characters)
- Control characters and invisible characters

### 2. Device Lifecycle Tests

**Purpose**: Verification of device operation completeness and reliability

#### 2.1 Comprehensive Lifecycle Tests

**File**: `tests/device_lifecycle_comprehensive_test.rs`  
**Test Count**: 16  
**Run Command**: `cargo test --test device_lifecycle_comprehensive_test`

**Coverage**:

- Device creation error scenarios (invalid data, duplicate names)
- State transition verification (Starting → Running → Stopped)
- Concurrent operation safety (simultaneous access control)
- Timeout handling and recovery
- Resource limitation handling (memory, storage)
- Device deletion and cleanup

**Implementation Example**:

```rust
#[tokio::test]
async fn test_device_operation_timeouts() {
    let mut app_state = create_test_app_state().await;
    // Simulate timeout scenarios
    for device in &mut app_state.android_devices {
        match device.status {
            DeviceStatus::Starting => {
                device.status = DeviceStatus::Error;
            }
        }
    }
}
```

#### 2.2 Device Operations & Status Tests

**File**: `tests/device_operations_status_test.rs`  
**Test Count**: Multiple  
**Run Command**: `cargo test --test device_operations_status_test`

**Coverage**:

- Device creation form validation
- Status update consistency
- Appropriate error state handling
- Device detail information accuracy

### 3. Performance Tests

**Purpose**: Maintaining performance standards and preventing performance regression

#### 3.1 Startup Performance Tests

**File**: `tests/startup_performance_test.rs`  
**Run Command**: `cargo test startup_performance_test -- --nocapture`

**Performance Standards**:

- Startup time: **< 150ms** (target)
- Current value: ~104ms ✅
- UI rendering: **< 50ms**
- Device loading: **< 100ms**

#### 3.2 Panel Switching Performance Tests

**File**: `tests/panel_switching_performance_test.rs`  
**Run Command**: `cargo test --test panel_switching_performance_test`

**Performance Standards**:

- Panel switching: **< 100ms**
- Focus movement: **< 50ms**
- Log updates: **< 25ms**
- Device detail updates: **< 25ms**

#### 3.3 Responsiveness Validation Tests

**File**: `tests/responsiveness_validation_test.rs`  
**Run Command**: `cargo test --test responsiveness_validation_test`

**Responsiveness Standards**:

- Keyboard input response: **< 8ms** (120fps)
- Event processing: **< 5ms**
- UI update frequency: **16.67ms** (60fps)

### 4. UI & Navigation Tests

**Purpose**: Verification of user interface accuracy and usability

#### 4.1 Focus & Theme Tests

**File**: `tests/ui_focus_and_theme_test.rs`  
**Run Command**: `cargo test --test ui_focus_and_theme_test`

**Coverage**:

- Focus state management (Android / iOS panels)
- Theme application accuracy
- Color themes and accessibility
- Keyboard navigation

#### 4.2 Device Creation Navigation Tests

**File**: `tests/device_creation_navigation_test.rs`  
**Run Command**: `cargo test --test device_creation_navigation_test`

**Coverage**:

- Form field navigation
- Circular navigation (first ↔ last)
- Field validation and error display
- Modal dialog control

### 5. Integration Tests

**Purpose**: System-wide operation verification and interaction testing

#### 5.1 Comprehensive Integration Tests

**File**: `tests/comprehensive_integration_test.rs`  
**Run Command**: `cargo test --test comprehensive_integration_test`

**Coverage**:

- Application-wide workflows
- Multi-component coordination
- End-to-end scenarios
- Platform-specific feature integration

### 6. Specialized Tests

#### 6.1 Command Security Tests

**File**: `tests/command_security_test.rs`  
**Run Command**: `cargo test --test command_security_test`

**Coverage**:

- Basic command injection prevention
- Safe command execution verification

## Test Execution Strategy

### Recommended Execution Order During Development

1. **Quick Check** (during development):

```bash
cargo test --bins --tests --quiet
```

2. **Security Verification** (feature completion):

```bash
cargo test security
```

3. **Performance Validation** (after optimization):

```bash
cargo test startup_performance_test -- --nocapture
```

4. **Comprehensive Verification** (before commit):

```bash
cargo test --bins --tests
```

### CI/CD Execution Patterns

GitHub Actions uses the following parallel execution pattern:

1. **test-coverage**: Coverage measurement
2. **security-tests**: Dedicated security test execution
3. **performance-tests**: Performance regression detection
4. **comprehensive-test-suite**: Full test execution

## Quality Indicators and Benchmarks

### Current Measurements

| Category      | Test Count | Execution Time  | Coverage  | Quality Level |
| ------------- | ---------- | --------------- | --------- | ------------- |
| Security      | 43+        | ~3 seconds      | 85%+      | High          |
| Lifecycle     | 16+        | ~4 seconds      | 75%+      | High          |
| Performance   | 6+         | ~2 seconds      | N/A       | High          |
| UI/Navigation | 15+        | ~6 seconds      | 60%+      | Medium-High   |
| Integration   | Multiple   | ~5 seconds      | 70%+      | High          |
| **Total**     | **200+**   | **~14 seconds** | **14.5%** | **High**      |

### Quality Standards

- ✅ **Test Success Rate**: 100% (all tests must pass)
- ✅ **Execution Time**: < 20 seconds (currently ~14 seconds)
- ✅ **Security Coverage**: Comprehensive protection implemented
- ✅ **Performance Standards**: All criteria met
- ✅ **Emulator Independence**: 100%

## Considerations for New Feature Development

### When Adding Security Features

- Make corresponding attack pattern tests mandatory
- Verify consistency with existing security tests
- Research and add new vulnerability patterns

### When Adding Device Features

- Include comprehensive lifecycle tests
- Verify comprehensive error handling
- Ensure concurrent processing safety

### When Adding UI Features

- Comply with accessibility requirements
- Ensure keyboard navigation completeness
- Verify behavior on different screen sizes

## References

- [Test Execution Guide](./test-execution-guide.md)
- [Test Addition Guidelines](./test-addition-guidelines.md)
- [Test-Driven Development Best Practices](https://martinfowler.com/bliki/TestDrivenDevelopment.html)
- [Security Testing Guide](https://owasp.org/www-project-web-security-testing-guide/)
