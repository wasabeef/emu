//! TestStateBuilder — decouple tests from AppState field layout.
//!
//! Tests should build state through this builder instead of touching
//! AppState fields directly. When AppState fields are renamed or
//! restructured, only this file needs updating.

use emu::app::state::{ApiLevelManagementState, AppState, FocusedPanel, Mode, Panel};
use emu::models::{AndroidDevice, IosDevice};

pub struct TestStateBuilder {
    state: AppState,
}

impl TestStateBuilder {
    /// Start with a default AppState (loading = false for test convenience).
    pub fn new() -> Self {
        let mut state = AppState::new();
        // Tests usually don't want the initial loading state
        state.is_loading = false;
        Self { state }
    }

    pub fn with_android_devices(mut self, devices: Vec<AndroidDevice>) -> Self {
        self.state.android_devices = devices;
        self
    }

    pub fn with_ios_devices(mut self, devices: Vec<IosDevice>) -> Self {
        self.state.ios_devices = devices;
        self
    }

    pub fn in_mode(mut self, mode: Mode) -> Self {
        self.state.mode = mode;
        self
    }

    pub fn on_panel(mut self, panel: Panel) -> Self {
        self.state.active_panel = panel;
        self
    }

    pub fn selecting_android(mut self, index: usize) -> Self {
        self.state.selected_android = index;
        self
    }

    pub fn selecting_ios(mut self, index: usize) -> Self {
        self.state.selected_ios = index;
        self
    }

    pub fn with_focus(mut self, focus: FocusedPanel) -> Self {
        self.state.focused_panel = focus;
        self
    }

    pub fn with_api_level_management(mut self, mgmt: ApiLevelManagementState) -> Self {
        self.state.api_level_management = Some(mgmt);
        self
    }

    pub fn loading(mut self, val: bool) -> Self {
        self.state.is_loading = val;
        self
    }

    /// Escape hatch for low-frequency fields (device_cache, log_task_handle, etc).
    /// Use sparingly — prefer adding a named builder method if used more than twice.
    pub fn with_raw(mut self, f: impl FnOnce(&mut AppState)) -> Self {
        f(&mut self.state);
        self
    }

    pub fn build(self) -> AppState {
        self.state
    }
}

impl Default for TestStateBuilder {
    fn default() -> Self {
        Self::new()
    }
}
