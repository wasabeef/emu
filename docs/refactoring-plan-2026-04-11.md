# Emu Refactoring Plan

Date: 2026-04-11

## Purpose

This document is a refined refactoring plan for the Emu codebase.
It is based on the current implementation, current tests, and a second-pass self-review of the first draft.

The goal is to make the codebase easier to change safely by:

- shrinking `App` into a thin orchestrator
- splitting `AndroidManager` and `IosManager` by responsibility
- breaking `platform -> app` dependency inversions
- keeping state logic cohesive and testable
- reducing high-risk giant files without changing behavior

## Behavior Preservation Contract

This refactor must be executed under a strict behavior-preservation contract.

Important note:

- no engineering process can honestly promise absolute mathematical proof of zero behavior change without a full formal specification
- this plan therefore uses the strongest practical guarantee available in this repository: behavior-preserving PR rules, characterization coverage, stable verification commands, and rollback checkpoints

For this project, "do not change behavior" means:

1. the same user inputs produce the same visible mode transitions
2. the same device operations trigger the same platform commands and state transitions
3. the same startup path produces the same loading, details, and log coordination behavior
4. the same cache read and write rules remain in effect
5. the same rendering contracts remain true for the existing test coverage
6. structural PRs do not intentionally change copy, ordering, command semantics, timing policy, or error handling

If any of the above changes, the PR is not structural-only and must be treated as a behavior change.

## Current Reality

The main hotspots are:

- [src/app/mod.rs](/Users/a12622/git/emu/src/app/mod.rs) `4317` lines
- [src/managers/android/mod.rs](/Users/a12622/git/emu/src/managers/android/mod.rs) `3924` lines before module extraction started
- [src/app/state/mod.rs](/Users/a12622/git/emu/src/app/state/mod.rs) `1668` lines before state module extraction started
- [src/ui/render.rs](/Users/a12622/git/emu/src/ui/render.rs) `1510` lines
- [src/managers/ios/mod.rs](/Users/a12622/git/emu/src/managers/ios/mod.rs) `1466` lines before module extraction started

These files currently hold multiple responsibilities at once.

## Progress Snapshot

The plan is now partially executed.

Completed structural checkpoints:

- `DeviceDetails` extracted from `app::state`
- `ApiLevelCache` extracted from `app::state`
- `src/app/state.rs` converted into `src/app/state/mod.rs` plus sibling modules
- app helper modules extracted so far:
  - `api_levels.rs`
  - `background.rs`
  - `details.rs`
  - `logs.rs`
  - `refresh.rs`
  - `input.rs`
  - `create_device.rs`
  - `device_actions.rs`
  - `tests.rs`
- app state helper modules extracted so far:
  - `ui.rs`
  - `logs.rs`
  - `cache.rs`
  - `api_levels.rs`
  - `details.rs`
  - `forms.rs`
  - `navigation.rs`
  - `notifications.rs`
  - `tests.rs`
- `src/managers/android.rs` converted into `src/managers/android/mod.rs`
- `src/managers/ios.rs` converted into `src/managers/ios/mod.rs`
- Android helper modules extracted so far:
  - `parser.rs`
  - `sdk.rs`
  - `version.rs`
  - `details.rs`
  - `create.rs`
  - `install.rs`
  - `discovery.rs`
  - `lifecycle.rs`
  - `tests.rs`
- iOS helper modules extracted so far:
  - `discovery.rs`
  - `details.rs`
  - `lifecycle.rs`
  - `tests.rs`
- UI helper modules extracted so far:
  - `dialogs/mod.rs`
  - `dialogs/create_device.rs`
  - `dialogs/confirmation.rs`
  - `dialogs/api_levels.rs`
  - `dialogs/notifications.rs`
  - `panels/mod.rs`
  - `panels/device_lists.rs`
  - `panels/details.rs`
  - `panels/logs.rs`
  - `panels/commands.rs`

Current file sizes after the latest structural checkpoints:

- [src/app/mod.rs](/Users/a12622/git/emu/src/app/mod.rs) `234` lines
- [src/app/tests.rs](/Users/a12622/git/emu/src/app/tests.rs) `1082` lines
- [src/managers/android/mod.rs](/Users/a12622/git/emu/src/managers/android/mod.rs) `517` lines
- [src/managers/android/tests.rs](/Users/a12622/git/emu/src/managers/android/tests.rs) `943` lines
- [src/app/state/mod.rs](/Users/a12622/git/emu/src/app/state/mod.rs) `337` lines
- [src/ui/render.rs](/Users/a12622/git/emu/src/ui/render.rs) is now the main rendering shell, with [src/ui/dialogs/mod.rs](/Users/a12622/git/emu/src/ui/dialogs/mod.rs) and [src/ui/panels/mod.rs](/Users/a12622/git/emu/src/ui/panels/mod.rs) already split out
- [src/models/device_info/mod.rs](/Users/a12622/git/emu/src/models/device_info/mod.rs) is now the `device_info` entrypoint, with [priority.rs](/Users/a12622/git/emu/src/models/device_info/priority.rs), [parsing.rs](/Users/a12622/git/emu/src/models/device_info/parsing.rs), and [tests.rs](/Users/a12622/git/emu/src/models/device_info/tests.rs) split out

Current review stance:

- completed extractions must continue to preserve behavior exactly
- any remaining structural split must justify its review value relative to churn
- all structural checkpoints must continue to pass targeted tests and `cargo clippy --all-targets --all-features -- -D warnings`
- any policy change, parsing correction, or fallback adjustment must stay in a separate behavior commit

## Dependency Inventory

This section now distinguishes between the original dependency problems and the current remaining ones.

### Historical inversions removed during this refactor

1. [src/managers/android/mod.rs](/Users/a12622/git/emu/src/managers/android/mod.rs)
   - no longer imports `crate::app::state::DeviceDetails`
   - no longer imports `crate::app::state::ApiLevelCache`

2. [src/managers/ios/mod.rs](/Users/a12622/git/emu/src/managers/ios/mod.rs)
   - no longer imports `crate::app::state::DeviceDetails`

3. [src/models/device_info/mod.rs](/Users/a12622/git/emu/src/models/device_info/mod.rs)
   - no longer keeps the old monolithic `device_info.rs` layout
   - device info tests now live in a dedicated module

### Current dependencies that are acceptable

1. [src/ui/render.rs](/Users/a12622/git/emu/src/ui/render.rs)
   - rendering from `AppState` is acceptable

2. [src/ui/widgets.rs](/Users/a12622/git/emu/src/ui/widgets.rs)
   - widget behavior depending on `Panel` is acceptable

These are not lower-layer inversions and do not currently justify further churn.

## Behavior Lock Scope

The following behaviors are explicitly locked during the refactor:

- startup sequence from `App::new()`
- background cache loading
- background device loading
- panel switching behavior
- device details refresh behavior
- log stream coordination behavior
- create device workflow behavior
- delete device workflow behavior
- wipe device workflow behavior
- Android target cache behavior
- Android and iOS device detail construction behavior
- current render contracts covered by existing tests

Any PR that changes one of these behaviors must be split out and treated as a behavior PR.

## Self-Review Findings

The first draft of the refactor plan had the right direction, but this review found several places where the implementation order needed to be tightened.

### Finding 1: top-level renames are too early

The earlier draft introduced `src/domain/`, `src/platform/`, and `src/state/` too early.
That would create large import churn before the underlying responsibilities are actually split.

Refined decision:

- keep current top-level module names during the first wave
- refactor inside existing roots first
- reconsider broad renames only after the structure is already stable

This means:

- keep `models/` for now
- keep `managers/` for now
- keep state under `app/state/`, not top-level `state/`

### Finding 2: moving `DeviceDetails` alone is not enough

At the start of the refactor, managers returned `crate::app::state::DeviceDetails`.
If `DeviceDetails` had been moved while still containing `app::state::Panel`, the dependency inversion would have remained.

Refined decision:

- move `DeviceDetails` out of `app::state`
- make its `platform` field use [src/models/platform.rs](/Users/a12622/git/emu/src/models/platform.rs) `Platform`
- avoid any `app` type inside the extracted details model

This is the first meaningful architecture break to make.

There was a second issue in the same area:

- `AndroidManager` read and wrote `ApiLevelCache` from `app::state`

So the real target was not "move one type", but:

- remove all `managers -> app::state` data-type dependencies that are not UI state

### Finding 3: `app/state.rs` should stay under `app`

The earlier draft pushed state into a top-level `src/state/`.
After review, that is not the best first move.

Reason:

- `AppState` is application state, not a reusable domain model
- `ui` rendering naturally depends on it
- keeping it under `app/` reduces churn and makes ownership clearer

Refined decision:

- split `src/app/state.rs` into `src/app/state/` submodules
- do not move it to a new top-level package in the first wave

### Finding 4: Rust file-to-directory conversions need their own PRs

These conversions are structural but noisy:

- `src/app/state.rs` -> `src/app/state/mod.rs`
- `src/managers/android.rs` -> `src/managers/android/mod.rs`
- `src/managers/ios.rs` -> `src/managers/ios/mod.rs`

If code extraction is mixed into the same PR, review quality drops.

Refined decision:

- do each file-to-directory conversion as a dedicated structural PR
- keep behavior identical in those PRs
- only then start extracting sibling modules

### Finding 5: `ui -> app::state` is not the main problem

The first draft treated `ui` importing `AppState` as something to remove quickly.
That is not actually the core issue.

`ui` rendering depending on application state is normal here.
The real problem is lower layers depending on higher layers.

Refined decision:

- prioritize removing `managers -> app` dependencies
- treat `ui -> app::state` cleanup as secondary and optional

### Finding 6: top-level package rename may not be worth the churn

Renaming `models -> domain` and `managers -> platform` might still be a good end-state.
But after self-review, this should be considered optional and late.

Refined decision:

- the first wave does not require any top-level package rename
- only revisit package renaming after the structural refactor proves valuable

### Finding 7: `Panel` and `Platform` must stay semantically separate

The codebase currently uses `Panel` both as a UI concept and as a quasi-platform marker in some places.
That is convenient, but it mixes concepts.

Refined decision:

- `Panel` remains a UI state enum
- `Platform` remains a domain/platform enum
- shared data models such as `DeviceDetails` must use `Platform`
- `AppState` can still track `active_panel: Panel`
- conversion between `Panel` and `Platform` must be explicit at the orchestration layer

This avoids leaking UI concepts into platform and model layers.

## Refactoring Principles

### 1. Structural changes and behavior changes stay separate

Each refactor PR should be one of:

- structural-only
- behavior-only

If a correctness fix is needed, it should be called out explicitly.

### 2. Break inverted dependencies before doing broad extraction

First remove:

- `managers -> app`
- `models tests -> app form types`

Do not start with large file moves before these are addressed.

### 3. Keep `emu::App` stable

The external usage should remain stable during the migration:

```rust
let app = App::new().await?;
app.run(terminal).await?;
```

### 4. Keep PRs reviewable

Target PR size should be small enough that:

- the moved responsibility is obvious
- reviewers can confirm behavior has not changed
- failures can be traced to one architectural move

### 5. Structural PRs must be observationally equivalent

For a structural PR, the expectation is observational equivalence under the current test and fixture surface.

That means:

- same inputs
- same outputs
- same state transitions
- same command-side effects

If a refactor requires changing expected outputs in a broad way, it is no longer a pure structural PR.

### 6. Keep tests close to the moved logic

When a module is extracted, move the nearest tests with it or create focused tests in `tests/`.

### 7. Prefer compatibility facades over giant one-shot rewrites

Temporary facades are acceptable:

- `app::state` re-exporting from `app/state/*`
- `managers::android` exposing the same facade while implementation moves underneath
- `ui::render` remaining as a small composition entrypoint while render functions move out

### 8. Protect semantic boundaries

Use these rules consistently:

- `Panel` is UI only
- `Platform` is domain/platform only
- persistent cache structures should not live under `app::state` if managers use them directly
- `models` may contain pure data and pure transformation logic, but not UI state

### 9. Refactor only behind fixed checkpoints

Every phase must complete behind a fixed checkpoint:

- clean working tree
- fixed verification commands
- no pending flaky tests
- one obvious rollback point

## Refined Target Architecture

This is the recommended first-wave structure.
It uses the existing top-level module names to reduce churn.

```text
src/
  main.rs
  lib.rs
  app/
    mod.rs
    runtime.rs
    dispatch.rs
    background.rs
    actions.rs
    selection.rs
    details.rs
    logs.rs
    state/
      mod.rs
      app_state.rs
      cache.rs
      forms.rs
      navigation.rs
      notifications.rs
      details.rs
      api_levels.rs
  managers/
    mod.rs
    common.rs
    android/
      mod.rs
      sdk.rs
      parser.rs
      avd.rs
      system_images.rs
      details.rs
      logcat.rs
      diagnostics.rs
    ios/
      mod.rs
      simctl.rs
      parser.rs
      details.rs
      logs.rs
  models/
    mod.rs
    device.rs
    device_info/
      mod.rs
      priority.rs
      parsing.rs
      tests.rs
    details.rs
    platform.rs
    error.rs
    api_level.rs
    device_naming.rs
  ui/
    mod.rs
    render.rs
    screen.rs
    layout.rs
    panels/
      mod.rs
      device_lists.rs
      details.rs
      logs.rs
      commands.rs
      details.rs
      logs.rs
      commands.rs
    dialogs/
      mod.rs
      create_device.rs
      confirm_delete.rs
      confirm_wipe.rs
      api_levels.rs
    theme.rs
    widgets.rs
  utils/
    ...
  constants/
    ...
```

## Module Responsibilities

### `app/`

Owns orchestration:

- application lifecycle
- event loop
- dispatch
- async coordination
- calling managers
- coordinating state and UI

### `app/state/`

Owns UI and interaction state:

- selected device indices
- panel focus
- forms
- notifications
- cached details
- log scroll state
- API level dialog state

### `managers/`

Owns platform interactions:

- command orchestration
- parsing SDK or simctl output
- device operations
- platform diagnostics
- platform detail construction

### `models/`

Owns shared data types and pure helpers:

- device models
- details model
- platform enum
- API level models
- pure naming helpers
- error types

It should not own:

- terminal UI state
- task handles
- direct filesystem cache policies unless they are truly shared

### `ui/`

Owns rendering only:

- layout
- screen composition
- panels
- dialogs
- formatting for display

## What Should Not Move Early

These are intentionally delayed:

- top-level package renames
- UI rendering redesign
- command behavior cleanup
- new abstractions around `DeviceManager` unless they remove a real pain point
- dependency injection changes not required by the current tests

## Core Invariants

These invariants should hold throughout the refactor:

1. `App::new()` and `App::run()` remain the public entrypoints
2. no behavior change is allowed in structural PRs
3. `managers` must move toward depending on `models` and `utils`, not `app`
4. `Panel` must not leak downward into shared model types
5. file-to-directory conversion PRs must not also do logic extraction
6. rename PRs must not also do behavior changes
7. structural PRs must not update broad expectation baselines
8. command invocation semantics must remain unchanged unless a behavior PR says otherwise

## Merge Gate for Structural PRs

A structural PR may be merged only if all of the following are true:

1. all standard verification commands pass
2. no intentionally changed expected outputs are introduced
3. no new integration path is added
4. no command arguments, cache policy, or platform branching logic changes
5. the reviewer can describe the PR as "same behavior, better ownership"

If one of these is false, the PR must be relabeled as a behavior PR or split.

## Required Verification Matrix

Every structural PR must run:

- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test -q`
- `RUST_TEST_THREADS=1 cargo test --bins --tests --features test-utils`

Additionally, the PR should run the narrowest relevant focused suite for the moved responsibility:

- state extraction PRs: relevant `app_state` tests
- `App` extraction PRs: `app` characterization tests
- Android manager PRs: Android manager and integration slices
- iOS manager PRs: iOS manager and integration slices
- UI PRs: render-oriented tests

The goal is not just green CI, but proving that the exact touched behavior surface stayed stable.

## Phase Plan

## Phase 0: Guardrails

### Objective

Ensure we can refactor aggressively without losing behavior guarantees.

### Required checks

- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test -q`
- `RUST_TEST_THREADS=1 cargo test --bins --tests --features test-utils`

### Exit criteria

- current suite is green
- characterization coverage exists for startup, details, logs, delete, wipe, panel switching
- verification commands are treated as mandatory merge gates, not suggestions

## Phase 1: Break the dependency inversions

### Objective

Remove the worst architecture violations before any broad extraction.

### PR 1A: Extract `DeviceDetails`

Files expected to change:

- [src/app/state/mod.rs](/Users/a12622/git/emu/src/app/state/mod.rs)
- [src/models/mod.rs](/Users/a12622/git/emu/src/models/mod.rs)
- new `src/models/details.rs`
- [src/managers/android/mod.rs](/Users/a12622/git/emu/src/managers/android/mod.rs)
- [src/managers/ios/mod.rs](/Users/a12622/git/emu/src/managers/ios/mod.rs)
- [src/ui/render.rs](/Users/a12622/git/emu/src/ui/render.rs)
- tests using `DeviceDetails`

Rules:

- `DeviceDetails.platform` must use `models::Platform`
- no `app` import inside `models/details.rs`
- no behavior change

Acceptance criteria:

- managers no longer import `crate::app::state::DeviceDetails`
- extracted model compiles without `app`
- all existing tests remain green
- no detail rendering expectation changes are required beyond import path adaptation

### PR 1B: Extract `ApiLevelCache` out of `app::state`

Files expected to change:

- [src/app/state/mod.rs](/Users/a12622/git/emu/src/app/state/mod.rs)
- [src/managers/android/mod.rs](/Users/a12622/git/emu/src/managers/android/mod.rs)
- new `src/utils/cache.rs` or other neutral cache module

Rules:

- `ApiLevelCache` must not live under `app`
- cache load/save semantics must remain identical
- no Android target listing behavior change

Acceptance criteria:

- `AndroidManager` no longer imports `crate::app::state::ApiLevelCache`
- cache file path and freshness behavior are unchanged
- cache-related tests stay green
- target listing order and display text remain unchanged

### PR 1C: Extract placeholder naming logic

Files expected to change:

- [src/app/state/mod.rs](/Users/a12622/git/emu/src/app/state/mod.rs)
- [src/models/device_info/mod.rs](/Users/a12622/git/emu/src/models/device_info/mod.rs)
- new `src/models/device_naming.rs` or equivalent pure helper

Rules:

- do not let `models` tests import `CreateDeviceForm`
- keep placeholder output identical

Acceptance criteria:

- `models/device_info` test no longer imports `CreateDeviceForm`
- placeholder generation behavior is unchanged
- no placeholder text expectation changes are required

## Phase 2: Convert `app/state.rs` into a directory module

### Objective

Make state extraction possible without mixing move noise and logic extraction.

### PR 2A: module conversion only

Files expected to change:

- `src/app/state.rs` -> `src/app/state/mod.rs`

Rules:

- identical code content as much as possible
- no extraction yet
- no behavior change

Acceptance criteria:

- build and tests are unchanged
- `app::state::*` import path still works
- diff is dominated by path movement, not logic edits

### PR 2B: extract cache and API level types

Files expected to change:

- `src/app/state/mod.rs`
- `src/app/state/cache.rs`
- `src/app/state/api_levels.rs`

### PR 2C: extract forms and notifications

Files expected to change:

- `src/app/state/mod.rs`
- `src/app/state/forms.rs`
- `src/app/state/notifications.rs`

### PR 2D: extract navigation and details cache helpers

Files expected to change:

- `src/app/state/mod.rs`
- `src/app/state/navigation.rs`
- `src/app/state/details.rs`
- `src/app/state/app_state.rs`

Acceptance criteria for Phase 2:

- `AppState` shell is visibly smaller
- form logic, notification logic, and cache logic are split
- `app::state` remains a stable import surface
- state submodules have single-responsibility names that match the methods they contain
- no externally visible state semantics change

## Phase 3: Thin `App`

### Objective

Reduce [src/app/mod.rs](/Users/a12622/git/emu/src/app/mod.rs) to a focused facade.

### PR 3A: extract background orchestration

Files expected to change:

- [src/app/mod.rs](/Users/a12622/git/emu/src/app/mod.rs)
- `src/app/background.rs`

Move:

- startup loading
- smart refresh scheduling
- cache warmup coordination

### PR 3B: extract details and log coordination

Files expected to change:

- [src/app/mod.rs](/Users/a12622/git/emu/src/app/mod.rs)
- `src/app/details.rs`
- `src/app/logs.rs`

Move:

- debounced detail scheduling
- detail fetching coordination
- log stream coordination
- stream readers

### PR 3C: extract dispatch and actions

Files expected to change:

- [src/app/mod.rs](/Users/a12622/git/emu/src/app/mod.rs)
- `src/app/runtime.rs`
- `src/app/dispatch.rs`
- `src/app/actions.rs`
- `src/app/selection.rs`

Move:

- event loop
- mode dispatch
- create, delete, wipe, toggle actions
- device and panel movement coordination

Acceptance criteria for Phase 3:

- `src/app/mod.rs` is under `1000` lines
- most implementation detail lives in sibling modules
- `App::new()` and `App::run()` stay stable
- the event loop and action handlers can be explained without scrolling through unrelated device code
- `App` characterization tests pass without expectation rewrites

## Phase 4: Convert `android.rs` into a directory module

### Objective

Prepare Android extraction without mixing module churn and logic churn.

### PR 4A: module conversion only

Files expected to change:

- `src/managers/android.rs` -> `src/managers/android/mod.rs`

Rules:

- keep the public `AndroidManager` facade intact
- no capability extraction in the same PR
- if API level is ambiguous, prefer `unknown` over inferred Android-version-to-API guesses
- if a fallback uses Android version text, allow only exact, unambiguous mappings and reject major-version guesses

### PR 4B: extract parser and SDK discovery

Files expected to change:

- `src/managers/android/mod.rs`
- `src/managers/android/parser.rs`
- `src/managers/android/sdk.rs`

### PR 4C: extract details and diagnostics

Files expected to change:

- `src/managers/android/mod.rs`
- `src/managers/android/details.rs`
- `src/managers/android/diagnostics.rs`

### PR 4D: extract AVD and system image operations

Files expected to change:

- `src/managers/android/mod.rs`
- `src/managers/android/avd.rs`
- `src/managers/android/system_images.rs`
- `src/managers/android/logcat.rs`

Acceptance criteria for Phase 4:

- Android manager facade is materially smaller
- parser logic no longer lives in the main facade file
- no single Android implementation file is over `1200` lines
- internal Android tests move with the extracted responsibility where feasible
- command behavior and parsing outputs remain unchanged

## Phase 5: Convert `ios.rs` into a directory module

### Objective

Apply the same extraction pattern to iOS.

### PR 5A: module conversion only

Files expected to change:

- `src/managers/ios.rs` -> `src/managers/ios/mod.rs`

### PR 5B: extract `simctl` integration and parser

Files expected to change:

- `src/managers/ios/mod.rs`
- `src/managers/ios/simctl.rs`
- `src/managers/ios/parser.rs`

### PR 5C: extract details and logs

Files expected to change:

- `src/managers/ios/mod.rs`
- `src/managers/ios/details.rs`
- `src/managers/ios/logs.rs`

Acceptance criteria for Phase 5:

- macOS-specific logic remains centralized
- non-macOS stub behavior is still obvious and safe
- iOS facade becomes small enough to read end-to-end
- `cfg(target_os = \"macos\")` usage does not spread uncontrollably across siblings
- existing iOS command and detail behavior remain unchanged

## Phase 6: Split UI rendering

### Objective

Reduce [src/ui/render.rs](/Users/a12622/git/emu/src/ui/render.rs) into smaller rendering modules without changing screen behavior.

### PR 6A: extract screen composition and layout

Files expected to change:

- [src/ui/render.rs](/Users/a12622/git/emu/src/ui/render.rs)
- `src/ui/screen.rs`
- `src/ui/layout.rs`

Status:

- not currently required
- after `dialogs` and `panels` extraction, [src/ui/render.rs](/Users/a12622/git/emu/src/ui/render.rs) is already a small rendering shell
- only revisit this if rendering orchestration grows again

### PR 6B: extract panels

Files expected to change:

- [src/ui/render.rs](/Users/a12622/git/emu/src/ui/render.rs)
- `src/ui/panels/mod.rs`
- `src/ui/panels/device_lists.rs`
- `src/ui/panels/details.rs`
- `src/ui/panels/logs.rs`
- `src/ui/panels/commands.rs`

Status:

- completed

### PR 6C: extract dialogs

Files expected to change:

- [src/ui/render.rs](/Users/a12622/git/emu/src/ui/render.rs)
- `src/ui/dialogs/mod.rs`
- optional later split:
- completed with dedicated dialog modules for create, confirmation, api levels, and notifications
  - `src/ui/dialogs/create_device.rs`
  - `src/ui/dialogs/confirm_delete.rs`
  - `src/ui/dialogs/confirm_wipe.rs`
  - `src/ui/dialogs/api_levels.rs`

Acceptance criteria for Phase 6:

- `ui::render::draw_app()` can remain as facade
- panel and dialog code live in dedicated files
- no rendering behavior changes
- render-oriented tests still assert the same visible output contracts
- no snapshot-style expectation rewrites except path-local module updates

## Phase 7: Optional package rename

### Objective

Only after the previous phases are green, decide whether top-level package rename is worth the churn.

Possible renames:

- `models` -> `domain`
- `managers` -> `platform`
- `utils` -> `infra`

This phase is optional.
It should happen only if the team still believes the rename improves comprehension enough to justify the diff noise.

## Phase 8: Bootstrap cleanup

### Objective

Reduce [src/main.rs](/Users/a12622/git/emu/src/main.rs) to assembly only.

Suggested files:

- `src/bootstrap/cli.rs`
- `src/bootstrap/logging.rs`
- `src/bootstrap/terminal.rs`

This is intentionally last because it is not a major source of maintenance pain right now.

## Recommended Execution Order

1. PR 1A: extract `DeviceDetails`
2. PR 1B: extract `ApiLevelCache`
3. PR 1C: extract placeholder naming helper
4. PR 2A: convert `app/state.rs` to `app/state/mod.rs`
5. PR 2B-2D: split state
6. PR 3A-3C: thin `App`
7. PR 4A-4D: split Android manager
8. PR 5A-5C: split iOS manager
9. PR 6A-6C: split UI
10. optional rename phase
11. bootstrap cleanup

## First Implementation Batch

The first implementation batch is ready now.

### Scope

- create `src/models/details.rs`
- move `DeviceDetails` out of `app::state`
- change `DeviceDetails.platform` to `models::Platform`
- update Android and iOS managers
- update `AppState` and `ui` imports
- do not touch `ApiLevelCache` in the same PR

### Why this first

- it is the clearest architecture win
- it has low surface area compared with the manager or UI splits
- it removes a real dependency inversion
- it unlocks later extraction work

### Batch 1 review checklist

- no `crate::app` import inside `src/models/details.rs`
- managers compile against `models::details::DeviceDetails`
- any conversion between `Platform` and panel state is explicit
- test expectations remain unchanged
- no command behavior changes

## PR Composition Rules

The following combinations are not allowed in the same PR:

1. file-to-directory conversion + logic extraction
2. module rename + behavior change
3. package rename + public API cleanup
4. UI extraction + manager extraction
5. event loop refactor + device operation refactor
6. structural refactor + cache policy change
7. structural refactor + command argument change

These combinations make review too noisy.

## Rollback Rules

If any PR causes one of the following, stop and reduce scope:

- full test suite becomes flaky again
- review diff is dominated by import churn rather than ownership change
- one PR changes more than one responsibility axis
- a structural PR requires wide expectation updates in tests
- a structural PR needs manual explanation for why behavior changed

When that happens:

- split the PR
- restore compatibility shims
- re-run the previous smaller checkpoint

## Success Metrics

This plan should be judged not only by “does it compile” but by whether the architecture becomes easier to work with.

Target metrics:

- `src/app/mod.rs` reduced from `4317` lines to under `1000`
- `src/managers/android/mod.rs` reduced from `3924` lines to a small facade plus focused siblings
- `src/app/state/mod.rs` converted into `app/state/*` with one main shell file
- no production lower-layer module importing `app::state::DeviceDetails`
- no production manager importing `app::state::ApiLevelCache`
- targeted tests for each extracted responsibility remain green
- no structural PR merges with changed behavior expectations

## PR Review Checklist

For every refactor PR, check:

- Is this PR structural-only?
- Does it reduce coupling, not just move code?
- Is the moved responsibility clearer afterward?
- Did it avoid introducing a new inversion?
- Did it keep existing behavior intact?
- Did it keep tests close to the moved logic?
- Is the module boundary now easier to explain than before?
- If this claims to be structural-only, did it keep all behavior baselines intact?

## Risks

### Risk: moving `DeviceDetails` forces wider enum changes

Yes. This is real.
But it is the right first problem to solve because the current inversion is architectural debt, not just file size debt.
Mitigation:

- require explicit `Panel <-> Platform` conversion helpers at orchestration boundaries
- forbid hidden implicit replacement of UI concepts in lower layers

### Risk: `ApiLevelCache` placement becomes awkward

Yes.
It is not a pure domain model, but it also should not live in `app::state`.

Current recommendation:

- place it in a neutral non-UI module such as `utils/cache.rs` first
- optimize the long-term home later if needed

Mitigation:

- lock cache file path and freshness tests before moving it
- do not redesign cache semantics in the extraction PR

### Risk: file-to-directory conversions create noisy diffs

Yes.
That is why they are isolated into dedicated structural PRs.

### Risk: UI extraction causes visual regressions

Yes.
That is why UI is later than state and manager extraction.
Mitigation:

- treat existing render tests as hard gates
- do not combine UI extraction with semantic display changes

### Risk: broad renames consume review budget

Yes.
That is why package renames are optional and late.

## Definition of Done

The refactor is done when:

- `App` is a thin orchestration facade
- `app/state/` is split by responsibility
- managers no longer depend on `app`
- shared models are owned by `models`
- Android and iOS manager facades are readable end-to-end
- UI rendering is separated by panel and dialog responsibility
- giant files are substantially reduced
- full test suite remains green through the migration

## Final Recommendation

Start with `PR 1A`.

Do not start with:

- top-level package renames
- UI extraction
- Android manager decomposition

Until `DeviceDetails` and the early state boundaries are cleaned up, those moves will create more churn than clarity.

If the team wants the strongest possible practical guarantee of no behavior change, every structural PR should be reviewed against the Behavior Preservation Contract above and rejected if it requires expectation rewrites outside of import-path adaptation.
