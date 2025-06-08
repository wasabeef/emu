//! Data models

pub mod device;
pub mod device_config;
pub mod device_info;
pub mod error;
pub mod platform;

pub use device::{AndroidDevice, DeviceStatus, IosDevice};
pub use error::DeviceError;
pub use platform::Platform;
