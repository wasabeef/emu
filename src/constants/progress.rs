//! Progress tracking constants for API installation and other operations.
//!
//! This module defines constants for progress tracking, particularly for
//! Android API level installation and system image management.
//!
//! # Progress Phases
//!
//! API installation typically follows these phases:
//! 1. Loading (0-20%) - Initial preparation
//! 2. Download (20-70%) - Downloading system images
//! 3. Extract (70-90%) - Extracting downloaded files
//! 4. Install (90-95%) - Installing to SDK location
//! 5. Cleanup (95-100%) - Finalizing installation
//!
//! # Increment Values
//!
//! Each phase uses different increment values to ensure smooth progress:
//! - Loading: 5% increments
//! - Download: 3% increments (for granular download progress)
//! - Extract: 4% increments
//! - Install: 5% increments
//! - Cleanup: 3% increments
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
pub const INSTALL_PHASE_INCREMENT: u8 = 5;
pub const CLEANUP_PHASE_INCREMENT: u8 = 3;
