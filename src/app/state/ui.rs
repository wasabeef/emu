/// Represents the two main device panels in the UI.
/// The application displays Android and iOS devices in separate panels.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Panel {
    /// Android device panel showing AVDs (Android Virtual Devices)
    Android,
    /// iOS device panel showing simulators (macOS only)
    Ios,
}

impl Panel {
    /// Toggles between Android and iOS panels.
    /// Returns the opposite panel from the current one.
    pub fn toggle(self) -> Self {
        match self {
            Self::Android => Self::Ios,
            Self::Ios => Self::Android,
        }
    }
}

/// Represents which UI panel currently has focus.
/// Used for keyboard navigation between device list and log area.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FocusedPanel {
    /// The device list panel (Android or iOS) has focus
    DeviceList,
    /// The log area panel has focus
    LogArea,
}

/// Application modes representing different UI states.
/// Each mode corresponds to a different screen or modal dialog.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Mode {
    /// Normal mode - viewing device lists
    Normal,
    /// Device creation modal is active
    CreateDevice,
    /// Delete confirmation dialog is active
    ConfirmDelete,
    /// Wipe data confirmation dialog is active
    ConfirmWipe,
    /// API level management dialog is active
    ManageApiLevels,
    /// Help screen is displayed
    Help,
}

/// Data for the delete confirmation dialog.
/// Stores information about the device to be deleted.
#[derive(Debug, Clone)]
pub struct ConfirmDeleteDialog {
    /// Display name of the device
    pub device_name: String,
    /// Unique identifier (AVD name for Android, UDID for iOS)
    pub device_identifier: String,
    /// Platform of the device being deleted
    pub platform: Panel,
}

/// Data for the wipe data confirmation dialog.
/// Stores information about the device whose data will be wiped.
#[derive(Debug, Clone)]
pub struct ConfirmWipeDialog {
    /// Display name of the device
    pub device_name: String,
    /// Unique identifier (AVD name for Android, UDID for iOS)
    pub device_identifier: String,
    /// Platform of the device being wiped
    pub platform: Panel,
}
