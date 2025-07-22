# Test Coverage Improvement Plan

## Current Status

- **Current Coverage**: 47.71% (5,173/10,842 lines)
- **Target Coverage**: 70.00% (7,589/10,842 lines)
- **Lines Needed**: 2,416 additional lines

## Strategic Approach

### Phase 1: High-Impact Files (Priority 1)

These files have the most uncovered lines and would provide the biggest coverage gains.

#### 1. `src/ui/render.rs` (0.0% coverage, 1,056 uncovered lines)

- **Impact**: Could add ~9.7% to overall coverage
- **Strategy**:
  - Test UI rendering logic with MockBackend
  - Focus on render_device_list, render_device_details, render_log_panel
  - Test modal rendering (device creation, confirmation dialogs)
- **Estimated effort**: High (complex UI logic)
- **Test files to create**: `tests/ui/render_test.rs`

#### 2. `src/app/state.rs` (21.4% coverage, 599 uncovered lines)

- **Impact**: Could add ~5.5% to overall coverage
- **Strategy**:
  - Test state transitions and mutations
  - Test cache management logic
  - Test device selection and navigation
  - Test background task coordination
- **Estimated effort**: Medium
- **Test files to create**: `tests/app/state_mutations_test.rs`

#### 3. `src/app/mod.rs` (26.7% coverage, 1,980 uncovered lines)

- **Impact**: Could improve significantly, but many lines may be UI interaction code
- **Strategy**:
  - Focus on testable business logic
  - Test action handlers
  - Test event processing
  - Skip terminal-specific code that's hard to test
- **Estimated effort**: High
- **Test files to enhance**: `tests/app/app_actions_test.rs`

### Phase 2: Medium-Impact Files (Priority 2)

#### 4. `src/managers/ios.rs` (47.5% coverage, 352 uncovered lines)

- **Impact**: Could add ~3.2% to overall coverage
- **Strategy**:
  - Test with MockCommandExecutor
  - Cover device lifecycle operations
  - Test error handling paths
- **Estimated effort**: Medium
- **Test files to create**: `tests/managers/ios_manager_test.rs`

#### 5. `src/managers/android.rs` (50.6% coverage, 1,160 uncovered lines)

- **Impact**: Already at 50%, but has many uncovered lines
- **Strategy**:
  - Focus on uncovered edge cases
  - Test error scenarios
  - Test device detail parsing
- **Estimated effort**: Medium
- **Test files to enhance**: Enhance existing tests

### Phase 3: Quick Wins (Priority 3)

#### 6. `src/app/event_processing.rs` (45.9% coverage, 72 uncovered lines)

- **Impact**: Small but easy to test
- **Strategy**: Test event batching and debouncing logic
- **Estimated effort**: Low

#### 7. `src/constants/defaults.rs` (0.0% coverage, 3 uncovered lines)

- **Impact**: Minimal but trivial to test
- **Strategy**: Simple doc tests
- **Estimated effort**: Trivial

## Execution Plan

### Week 1: UI Testing Infrastructure

1. Implement comprehensive tests for `src/ui/render.rs`
2. Expected coverage gain: +9.7% (57.4% total)

### Week 2: State Management

1. Complete tests for `src/app/state.rs`
2. Test iOS manager with mocks
3. Expected coverage gain: +8.7% (66.1% total)

### Week 3: Final Push

1. Add targeted tests for `src/app/mod.rs` (testable parts)
2. Fill in remaining gaps
3. Expected coverage gain: +3.9% (70.0% total)

## Key Testing Strategies

### 1. Use MockBackend for UI Tests

```rust
let mut terminal = Terminal::new(MockBackend::new(80, 40)).unwrap();
// Test rendering without actual terminal
```

### 2. Use MockCommandExecutor for Manager Tests

```rust
let mock_executor = MockCommandExecutor::new()
    .with_success("xcrun", &["simctl", "list"], "...");
```

### 3. Focus on Business Logic

- Prioritize testing pure functions and logic
- Skip terminal-specific code that's hard to test
- Use integration tests for end-to-end scenarios

### 4. Test Error Paths

- Many uncovered lines are in error handling
- Use mocks to trigger error conditions
- Ensure all error types are covered

## Success Metrics

- Reach 70% overall coverage
- No file below 40% coverage (except constants)
- All critical business logic above 80% coverage
- Maintain test execution time under 5 minutes
