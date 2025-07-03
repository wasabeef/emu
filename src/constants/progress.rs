/// Progress tracking constants for API installation and other operations
// API installation progress phase thresholds
pub const DOWNLOAD_PHASE_START_PERCENTAGE: u8 = 20;
pub const EXTRACT_PHASE_START_PERCENTAGE: u8 = 70;
pub const INSTALL_PHASE_START_PERCENTAGE: u8 = 90;
pub const COMPLETION_THRESHOLD_PERCENTAGE: u8 = 95;

// Progress calculation constants
pub const DOWNLOAD_PROGRESS_MULTIPLIER: u8 = 50;
pub const DOWNLOAD_PROGRESS_DIVISOR: u8 = 100;

// Additional progress phase values
pub const PROGRESS_PHASE_75_PERCENT: u8 = 75;
pub const PROGRESS_PHASE_85_PERCENT: u8 = 85;
pub const PROGRESS_PHASE_100_PERCENT: u8 = 100;

// Progress increment values for each phase
pub const LOADING_PHASE_INCREMENT: u8 = 5;
pub const DOWNLOAD_PHASE_INCREMENT: u8 = 3;
pub const EXTRACT_PHASE_INCREMENT: u8 = 4;
