//! Device managers module

pub mod android;
pub mod common;
pub mod ios;

pub use android::AndroidManager;
pub use ios::IosManager;
