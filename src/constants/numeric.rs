/// Numeric constants for calculations and conversions
// Unit conversion constants (correct values)
pub const BYTES_PER_KB: u64 = 1024;
pub const BYTES_PER_MB: u64 = 1024 * 1024;
pub const BYTES_PER_GB: u64 = 1024 * 1024 * 1024;

// Version parsing divisors
pub const VERSION_MAJOR_DIVISOR: f32 = 10.0;
pub const VERSION_MINOR_DIVISOR: f32 = 100.0;
pub const VERSION_PATCH_DIVISOR: f32 = 10000.0;

// Default version value
pub const VERSION_DEFAULT: f32 = 0.0;

// iOS device batch processing
pub const IOS_DEVICE_PARSE_BATCH_SIZE: usize = 10;
