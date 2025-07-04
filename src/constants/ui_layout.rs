/// UI layout constants for panel sizes and dimensions
// Panel percentage constants for three-panel layout
pub const DEVICE_PANELS_PERCENTAGE: u16 = 60; // Combined Android + iOS panels
pub const ANDROID_PANEL_PERCENTAGE: u16 = 30;
pub const IOS_PANEL_PERCENTAGE: u16 = 30;
pub const DEVICE_DETAILS_PANEL_PERCENTAGE: u16 = 40;

// Panel switch delay
pub const PANEL_SWITCH_DELAY_MS: u64 = 50;

// Dialog dimensions
pub const DIALOG_WIDTH_SMALL: u16 = 60;
pub const DIALOG_HEIGHT_SMALL: u16 = 10;
pub const DIALOG_WIDTH_MEDIUM: u16 = 80;
pub const DIALOG_HEIGHT_MEDIUM: u16 = 16;
pub const DIALOG_WIDTH_LARGE: u16 = 90;
pub const DIALOG_HEIGHT_LARGE: u16 = 26;

// Minimum terminal dimensions
pub const MIN_TERMINAL_WIDTH: u16 = 40;
pub const MIN_TERMINAL_HEIGHT: u16 = 10;

// Form and display constraints
pub const FORM_LABEL_WIDTH: u16 = 20;
pub const API_LEVEL_LIST_MIN_HEIGHT: u16 = 15;
pub const SEPARATOR_LENGTH: u16 = 30;

// Character display limits
pub const DEVICE_TYPE_DISPLAY_MAX_LENGTH: usize = 25;
pub const DEVICE_TYPE_TRUNCATED_LENGTH: usize = 22;
pub const ERROR_MESSAGE_TRUNCATED_LENGTH: usize = 147;

// Dialog and UI element margins
pub const DIALOG_MARGIN: u16 = 4;
pub const LOADING_INDICATOR_MARGIN: u16 = 3;

// Log display dimensions
pub const LOG_TIMESTAMP_WIDTH: usize = 9;
pub const LOG_LEVEL_WIDTH: usize = 9;
pub const MESSAGE_TRUNCATE_SUFFIX_LENGTH: usize = 3;

// Notification dimensions
pub const NOTIFICATION_HEIGHT: u16 = 4;

// Header and status bar heights
pub const HEADER_HEIGHT: u16 = 3;
pub const STATUS_BAR_HEIGHT: u16 = 1;

// Form footer heights
pub const FORM_FOOTER_HEIGHT: u16 = 3;

// Additional UI layout constants
pub const DIALOG_MIN_WIDTH: u16 = 40;
pub const DIALOG_MIN_HEIGHT: u16 = 8;
pub const FORM_FIELD_WIDTH: u16 = 30;
pub const PANEL_MIN_WIDTH: u16 = 20;
pub const PANEL_MIN_HEIGHT: u16 = 5;
pub const TAB_STOP: u16 = 4;

// Padding constants
pub const DEFAULT_PADDING: u16 = 1;
pub const LIST_ITEM_PADDING: u16 = 1;
pub const WIDGET_PADDING: u16 = 1;
pub const VERTICAL_SPACING: u16 = 1;
pub const SECTION_SPACING: u16 = 2;

// Border constants
pub const BORDER_WIDTH: u16 = 1;
pub const FOCUS_BORDER_WIDTH: u16 = 2;

// List display constants
pub const MAX_VISIBLE_ITEMS: u16 = 20;
pub const MIN_VISIBLE_ITEMS: u16 = 5;
pub const PAGE_SIZE: u16 = 10;
pub const SCROLL_OFFSET: u16 = 3;

// Animation timing constants
pub const LOADING_ANIMATION_INTERVAL_MS: u64 = 100;
pub const SPINNER_FRAME_DURATION_MS: u64 = 100;
pub const NOTIFICATION_DURATION_MS: u64 = 3000;
