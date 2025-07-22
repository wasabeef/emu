# Test Coverage Action Plan - Path to 70%

## Executive Summary

**Current**: 47.71% (5,173/10,842 lines)  
**Target**: 70.00% (7,589/10,842 lines)  
**Gap**: 2,416 lines needed

## Phase 1: UI Rendering Tests (Week 1)

**Target File**: `src/ui/render.rs` (0% → 80%)  
**Impact**: +7.8% overall coverage (844 lines)  
**Test File**: Create `tests/unit/ui/render_functions_test.rs`

### Functions to Test

1. **Panel Rendering** (300 lines)
   - `render_android_panel` - List view with device status
   - `render_ios_panel` - List view with iOS devices
   - `render_device_details_panel` - Device properties display

2. **Modal/Dialog Rendering** (400 lines)
   - `render_create_device_dialog` - Form fields and validation
   - `render_api_level_dialog` - API level selection
   - `render_confirmation_dialog` - Delete/wipe confirmations

3. **Component Rendering** (144 lines)
   - `render_notifications` - Toast messages
   - `render_log_panel` - Log viewer with filtering
   - `render_device_commands` - Action buttons

### Test Strategy

```rust
// Example test structure
#[test]
fn test_render_android_panel_with_devices() {
    let backend = MockBackend::new(120, 40);
    let mut terminal = Terminal::new(backend).unwrap();

    // Setup test data
    let devices = vec![test_android_device()];
    let state = test_state_with_devices(devices);

    // Call specific render function
    terminal.draw(|f| {
        let area = f.size();
        render_android_panel(f, area, &state, &theme);
    }).unwrap();

    // Verify output
    assert!(backend.assert_contains_text("Test Device"));
}
```

## Phase 2: State Management (Week 2)

**Target File**: `src/app/state.rs` (21.4% → 70%)  
**Impact**: +3.7% overall coverage (370 lines)  
**Test File**: Create `tests/unit/app/state_operations_test.rs`

### Functions to Test

1. **Cache Management** (150 lines)
   - `is_cache_available`
   - `populate_form_from_cache`
   - `DeviceCache::is_stale`

2. **Device State Updates** (120 lines)
   - `update_single_android_device_status`
   - `update_single_ios_device_status`
   - `get_selected_device_details`

3. **Form Navigation** (100 lines)
   - `CreateDeviceForm::next_field`
   - `CreateDeviceForm::prev_field`
   - Field validation logic

## Phase 3: iOS Manager (Week 2)

**Target File**: `src/managers/ios.rs` (47.5% → 75%)  
**Impact**: +1.8% overall coverage (185 lines)  
**Test File**: Create `tests/unit/managers/ios_operations_test.rs`

### Functions to Test

1. **Device Operations** (100 lines)
   - `list_device_types`
   - `get_device_details`
   - `erase_device`

2. **Lifecycle Management** (85 lines)
   - `start_device` with Simulator.app handling
   - Error handling paths

## Phase 4: Quick Wins (Week 3)

**Multiple Small Files**  
**Impact**: +1.2% overall coverage (130 lines)

1. **Event Processing** (72 lines)
   - Test `NavigationBatcher` and `EventDebouncer`
2. **Command Executor** (35 lines)
   - Error paths and retry logic

3. **Constants** (3 lines)
   - Simple doc tests

## Implementation Guidelines

### 1. Use Existing Test Infrastructure

```rust
// Leverage MockBackend for UI tests
let backend = MockBackend::new(width, height);

// Use MockCommandExecutor for managers
let mock = MockCommandExecutor::new()
    .with_success("xcrun", &["simctl", "list"], "...");
```

### 2. Focus on Business Logic

- Skip terminal-specific code
- Test data transformations
- Cover error handling paths
- Validate state transitions

### 3. Test Organization

```
tests/
├── unit/
│   ├── ui/
│   │   ├── render_functions_test.rs    # NEW
│   │   └── render_helper_test.rs       # Existing
│   ├── app/
│   │   ├── state_operations_test.rs    # NEW
│   │   └── state_test.rs               # Existing
│   └── managers/
│       └── ios_operations_test.rs      # NEW
└── integration/
    └── ui_render_test.rs               # Existing
```

## Success Metrics

### Weekly Targets

- **Week 1**: 55% coverage (+7.3%)
- **Week 2**: 65% coverage (+10%)
- **Week 3**: 70% coverage (+5%)

### File-Level Targets

| File                    | Current | Target | Lines Needed |
| ----------------------- | ------- | ------ | ------------ |
| ui/render.rs            | 0%      | 80%    | 844          |
| app/state.rs            | 21.4%   | 70%    | 370          |
| managers/ios.rs         | 47.5%   | 75%    | 185          |
| app/event_processing.rs | 45.9%   | 80%    | 45           |

## Execution Commands

```bash
# Run coverage analysis
cargo llvm-cov --lcov --output-path coverage/lcov.info --features test-utils \
  --ignore-filename-regex '(tests/|src/main\.rs|src/bin/|src/app/test_helpers\.rs|src/fixtures/|src/managers/mock\.rs)'

# Monitor progress
cargo llvm-cov --features test-utils --html

# Run specific test suites
cargo test --test render_functions_test --features test-utils
cargo test --test state_operations_test --features test-utils
cargo test --test ios_operations_test --features test-utils
```

## Risk Mitigation

1. **Complex UI Logic**: Use MockBackend's text extraction for assertions
2. **Async Testing**: Use tokio::test macro consistently
3. **State Isolation**: Create fresh fixtures for each test
4. **Performance**: Keep test execution under 5 minutes

This focused approach targets the highest-impact files first, providing the most efficient path to 70% coverage.
